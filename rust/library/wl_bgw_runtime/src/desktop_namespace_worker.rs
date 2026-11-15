//! Isolated, bounded STA worker for Shell Desktop namespace snapshots.
//!
//! A slow or stuck namespace provider can delay this worker, but never the BGW
//! Explorer-monitor loop. At most one request is in flight; the host never
//! spawns replacement workers after a timeout.

use std::io::{Error, ErrorKind};
use std::sync::mpsc::{
    self, Receiver, RecvTimeoutError, SyncSender, TryRecvError, TrySendError,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use wl_wlde_win32::{
    enumerate_desktop_namespace_items, DesktopNamespaceSnapshot,
};

const REQUEST_IDLE_WAIT: Duration = Duration::from_millis(50);
const RESULT_RETRY_WAIT: Duration = Duration::from_millis(10);
const NAMESPACE_WORK_TIMEOUT: Duration = Duration::from_secs(20);
const COINIT_APARTMENTTHREADED: u32 = 0x2;

#[cfg(windows)]
#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(reserved: *mut std::ffi::c_void, concurrency_model: u32) -> i32;
    fn CoUninitialize();
}

#[repr(C)]
struct NativePoint {
    x: i32,
    y: i32,
}

#[repr(C)]
struct NativeMessage {
    hwnd: *mut std::ffi::c_void,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    point: NativePoint,
    private: u32,
}

#[link(name = "user32")]
extern "system" {
    fn PeekMessageW(
        message: *mut NativeMessage,
        window: *mut std::ffi::c_void,
        minimum: u32,
        maximum: u32,
        remove: u32,
    ) -> i32;
    fn TranslateMessage(message: *const NativeMessage) -> i32;
    fn DispatchMessageW(message: *const NativeMessage) -> isize;
}

const PM_REMOVE: u32 = 0x0001;

fn pump_sta_messages() {
    loop {
        let mut message = std::mem::MaybeUninit::<NativeMessage>::uninit();
        let available = unsafe {
            PeekMessageW(
                message.as_mut_ptr(),
                std::ptr::null_mut(),
                0,
                0,
                PM_REMOVE,
            )
        };
        if available == 0 {
            break;
        }
        let message = unsafe { message.assume_init() };
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

struct ShellComApartment;

impl ShellComApartment {
    fn initialize() -> Result<Self, i32> {
        let result = unsafe {
            CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED)
        };
        if result < 0 {
            Err(result)
        } else {
            // S_OK and S_FALSE both require one matching CoUninitialize.
            Ok(Self)
        }
    }
}

impl Drop for ShellComApartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

struct RefreshRequest {
    generation: u64,
}

struct RefreshReply {
    generation: u64,
    result: Result<DesktopNamespaceSnapshot, String>,
}

fn send_reply(
    sender: &SyncSender<RefreshReply>,
    mut reply: RefreshReply,
) -> bool {
    loop {
        pump_sta_messages();
        match sender.try_send(reply) {
            Ok(()) => return true,
            Err(TrySendError::Full(returned)) => {
                reply = returned;
                thread::sleep(RESULT_RETRY_WAIT);
            }
            Err(TrySendError::Disconnected(_)) => return false,
        }
    }
}

#[derive(Debug)]
pub enum DesktopNamespaceWorkerPoll {
    Ready {
        generation: u64,
        snapshot: DesktopNamespaceSnapshot,
    },
    Failed {
        generation: u64,
        message: String,
    },
    TimedOut {
        generation: u64,
    },
}

/// A single STA worker with one bounded request and one bounded result slot.
pub struct DesktopNamespaceWorker {
    request_tx: SyncSender<RefreshRequest>,
    result_rx: Receiver<RefreshReply>,
    pending_generation: Option<u64>,
    requested_at: Option<Instant>,
    timeout_reported: bool,
    _thread: JoinHandle<()>,
}

impl DesktopNamespaceWorker {
    pub fn new() -> std::io::Result<Self> {
        let (request_tx, request_rx) = mpsc::sync_channel::<RefreshRequest>(1);
        let (result_tx, result_rx) = mpsc::sync_channel::<RefreshReply>(1);
        let (startup_tx, startup_rx) = mpsc::sync_channel::<Result<(), i32>>(1);

        let worker = thread::Builder::new()
            .name("winlook-wlde-shell-namespace".to_owned())
            .spawn(move || {
                let _apartment = match ShellComApartment::initialize() {
                    Ok(apartment) => {
                        if startup_tx.send(Ok(())).is_err() {
                            return;
                        }
                        apartment
                    }
                    Err(code) => {
                        let _ = startup_tx.send(Err(code));
                        return;
                    }
                };

                loop {
                    pump_sta_messages();
                    let request = match request_rx.recv_timeout(REQUEST_IDLE_WAIT) {
                        Ok(request) => request,
                        Err(RecvTimeoutError::Timeout) => continue,
                        Err(RecvTimeoutError::Disconnected) => break,
                    };

                    pump_sta_messages();
                    let result = enumerate_desktop_namespace_items()
                        .map_err(|error| error.to_string());
                    if !send_reply(
                        &result_tx,
                        RefreshReply {
                            generation: request.generation,
                            result,
                        },
                    ) {
                        break;
                    }
                }
            })?;

        match startup_rx.recv() {
            Ok(Ok(())) => {}
            Ok(Err(code)) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Shell namespace worker COM initialization failed: {code:#x}"),
                ));
            }
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::BrokenPipe,
                    "Shell namespace worker exited before STA initialization completed",
                ));
            }
        }

        Ok(Self {
            request_tx,
            result_rx,
            pending_generation: None,
            requested_at: None,
            timeout_reported: false,
            _thread: worker,
        })
    }

    /// Queues one refresh for the current verified Explorer-host generation.
    /// Returns false while an older request is still in flight or the worker
    /// has exited. It never creates another worker.
    pub fn request_refresh(&mut self, generation: u64) -> bool {
        if self.pending_generation.is_some() {
            return false;
        }

        match self.request_tx.try_send(RefreshRequest { generation }) {
            Ok(()) => {
                self.pending_generation = Some(generation);
                self.requested_at = Some(Instant::now());
                self.timeout_reported = false;
                true
            }
            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => false,
        }
    }

    /// Retrieves a completed snapshot without blocking the BGW host loop.
    /// A timeout is reported once; the in-flight worker is not killed or
    /// replaced, because Rust cannot safely terminate a blocked Shell call.
    pub fn poll(&mut self) -> Option<DesktopNamespaceWorkerPoll> {
        match self.result_rx.try_recv() {
            Ok(reply) => {
                self.pending_generation = None;
                self.requested_at = None;
                self.timeout_reported = false;
                return Some(match reply.result {
                    Ok(snapshot) => DesktopNamespaceWorkerPoll::Ready {
                        generation: reply.generation,
                        snapshot,
                    },
                    Err(message) => DesktopNamespaceWorkerPoll::Failed {
                        generation: reply.generation,
                        message,
                    },
                });
            }
            Err(TryRecvError::Disconnected) => {
                if let Some(generation) = self.pending_generation.take() {
                    self.requested_at = None;
                    return Some(DesktopNamespaceWorkerPoll::Failed {
                        generation,
                        message: "Shell namespace worker exited unexpectedly".to_owned(),
                    });
                }
                return None;
            }
            Err(TryRecvError::Empty) => {}
        }

        let generation = self.pending_generation?;
        let requested_at = self.requested_at?;
        if !self.timeout_reported && requested_at.elapsed() >= NAMESPACE_WORK_TIMEOUT {
            self.timeout_reported = true;
            return Some(DesktopNamespaceWorkerPoll::TimedOut { generation });
        }
        None
    }

    pub fn has_pending_work(&self) -> bool {
        self.pending_generation.is_some()
    }
}
