//! Isolated, bounded STA worker for Shell Desktop namespace snapshots.
//!
//! A slow or stuck namespace provider can delay this worker, but never the BGW
//! Explorer-monitor loop. At most one request is in flight; the host never
//! spawns replacement workers after a timeout.

use std::io::{Error, ErrorKind};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::desktop_sta::StaWakeEvent;
use wl_wlde_win32::{
    enumerate_desktop_namespace_items, probe_desktop_folder_view, DesktopHostIdentity,
    DesktopNamespaceSnapshot, DesktopViewProbe,
};

const REQUEST_IDLE_WAIT: Duration = Duration::from_millis(50);
const RESULT_RETRY_WAIT: Duration = Duration::from_millis(10);
const NAMESPACE_WORK_TIMEOUT: Duration = Duration::from_secs(20);
const COINIT_APARTMENTTHREADED: u32 = 0x2;

#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(reserved: *mut std::ffi::c_void, concurrency_model: u32) -> i32;
    fn CoUninitialize();
}

struct ShellComApartment;

impl ShellComApartment {
    fn initialize() -> Result<Self, i32> {
        let result = unsafe { CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED) };
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
    host_identity: DesktopHostIdentity,
}

struct RefreshReply {
    generation: u64,
    result: Result<DesktopNamespaceSnapshot, String>,
    view_result: Result<DesktopViewProbe, String>,
}

fn send_reply(
    sender: &SyncSender<RefreshReply>,
    mut reply: RefreshReply,
    wake_event: &StaWakeEvent,
) -> bool {
    loop {
        match sender.try_send(reply) {
            Ok(()) => return true,
            Err(TrySendError::Full(returned)) => {
                reply = returned;
                if wake_event.wait_and_pump(RESULT_RETRY_WAIT).is_err() {
                    return false;
                }
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
        view_result: Result<DesktopViewProbe, String>,
    },
    Failed {
        generation: u64,
        message: String,
        view_result: Result<DesktopViewProbe, String>,
    },
    TimedOut {
        generation: u64,
    },
}

/// A single STA worker with one bounded request and one bounded result slot.
pub struct DesktopNamespaceWorker {
    request_tx: SyncSender<RefreshRequest>,
    result_rx: Receiver<RefreshReply>,
    wake_event: StaWakeEvent,
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
        let wake_event = StaWakeEvent::new()?;
        let worker_wake_event = wake_event.clone();

        let worker = thread::Builder::new()
            .name("winlook-wlde-shell-namespace".to_owned())
            .spawn(move || {
                let _apartment = match ShellComApartment::initialize() {
                    Ok(apartment) => apartment,
                    Err(code) => {
                        let _ = startup_tx.send(Err(code));
                        return;
                    }
                };

                if startup_tx.send(Ok(())).is_err() {
                    return;
                }

                // Ensure the STA message queue exists before the first wait.
                worker_wake_event.pump_messages();
                loop {
                    if worker_wake_event.wait_and_pump(REQUEST_IDLE_WAIT).is_err() {
                        break;
                    }

                    let request = match request_rx.try_recv() {
                        Ok(request) => request,
                        Err(TryRecvError::Empty) => continue,
                        Err(TryRecvError::Disconnected) => break,
                    };

                    // Probe the live Desktop view first. Both Shell operations stay on
                    // this STA and return only owned, path-free data to BGW.
                    let view_result = probe_desktop_folder_view(&request.host_identity)
                        .map_err(|error| error.to_string());
                    let result =
                        enumerate_desktop_namespace_items().map_err(|error| error.to_string());
                    if !send_reply(
                        &result_tx,
                        RefreshReply {
                            generation: request.generation,
                            result,
                            view_result,
                        },
                        &worker_wake_event,
                    ) {
                        break;
                    }
                }
            })?;

        match startup_rx.recv() {
            Ok(Ok(())) => {}
            Ok(Err(code)) => {
                return Err(Error::other(format!(
                    "Shell namespace worker COM initialization failed: {code:#x}"
                )));
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
            wake_event,
            pending_generation: None,
            requested_at: None,
            timeout_reported: false,
            _thread: worker,
        })
    }

    /// Queues one refresh for the current verified Explorer-host generation.
    /// Returns false while an older request is still in flight or the worker
    /// has exited. It never creates another worker.
    pub fn request_refresh(
        &mut self,
        generation: u64,
        host_identity: DesktopHostIdentity,
    ) -> bool {
        if self.pending_generation.is_some() {
            return false;
        }

        match self.request_tx.try_send(RefreshRequest {
            generation,
            host_identity,
        }) {
            Ok(()) => {
                self.pending_generation = Some(generation);
                self.requested_at = Some(Instant::now());
                self.timeout_reported = false;
                self.wake_event.signal();
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
                        view_result: reply.view_result,
                    },
                    Err(message) => DesktopNamespaceWorkerPoll::Failed {
                        generation: reply.generation,
                        message,
                        view_result: reply.view_result,
                    },
                });
            }
            Err(TryRecvError::Disconnected) => {
                if let Some(generation) = self.pending_generation.take() {
                    self.requested_at = None;
                    return Some(DesktopNamespaceWorkerPoll::Failed {
                        generation,
                        message: "Shell namespace worker exited unexpectedly".to_owned(),
                        view_result: Err("Shell namespace worker exited unexpectedly".to_owned()),
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

impl Drop for DesktopNamespaceWorker {
    fn drop(&mut self) {
        // Wake the STA so it can observe channel disconnection.
        self.wake_event.signal();
    }
}
