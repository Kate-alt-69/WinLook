//! Isolated, bounded STA worker for read-only live Explorer Desktop view probes.
//!
//! A Shell/Explorer COM call may stall this worker, but it never blocks the
//! BGW host monitor or the independent Desktop namespace inventory worker.
//! At most one request is in flight; the host never spawns replacements after
//! a timeout.

use std::io::{Error, ErrorKind};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::desktop_sta::StaWakeEvent;
use wl_wlde_win32::{probe_desktop_folder_view, DesktopHostIdentity, DesktopViewProbe};

const REQUEST_IDLE_WAIT: Duration = Duration::from_millis(50);
const RESULT_RETRY_WAIT: Duration = Duration::from_millis(10);
const DESKTOP_VIEW_PROBE_TIMEOUT: Duration = Duration::from_secs(20);
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

struct ProbeRequest {
    generation: u64,
    host_identity: DesktopHostIdentity,
}

struct ProbeReply {
    generation: u64,
    result: Result<DesktopViewProbe, String>,
}

fn send_reply(
    sender: &SyncSender<ProbeReply>,
    mut reply: ProbeReply,
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
pub enum DesktopViewWorkerPoll {
    Ready {
        generation: u64,
        probe: DesktopViewProbe,
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
pub struct DesktopViewWorker {
    request_tx: SyncSender<ProbeRequest>,
    result_rx: Receiver<ProbeReply>,
    wake_event: StaWakeEvent,
    pending_generation: Option<u64>,
    requested_at: Option<Instant>,
    timeout_reported: bool,
    _thread: JoinHandle<()>,
}

impl DesktopViewWorker {
    pub fn new() -> std::io::Result<Self> {
        let (request_tx, request_rx) = mpsc::sync_channel::<ProbeRequest>(1);
        let (result_tx, result_rx) = mpsc::sync_channel::<ProbeReply>(1);
        let (startup_tx, startup_rx) = mpsc::sync_channel::<Result<(), i32>>(1);
        let wake_event = StaWakeEvent::new()?;
        let worker_wake_event = wake_event.clone();

        let worker = thread::Builder::new()
            .name("winlook-wlde-desktop-view".to_owned())
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

                    let result = probe_desktop_folder_view(&request.host_identity)
                        .map_err(|error| error.to_string());
                    if !send_reply(
                        &result_tx,
                        ProbeReply {
                            generation: request.generation,
                            result,
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
                    "Desktop view worker COM initialization failed: {code:#x}"
                )));
            }
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::BrokenPipe,
                    "Desktop view worker exited before STA initialization completed",
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

    /// Queues one view probe for the currently verified Explorer-host generation.
    /// It returns false while an older probe is in flight or the worker has exited.
    pub fn request_probe(&mut self, generation: u64, host_identity: DesktopHostIdentity) -> bool {
        if self.pending_generation.is_some() {
            return false;
        }

        match self.request_tx.try_send(ProbeRequest {
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

    /// Retrieves a completed probe without blocking the BGW host loop.
    /// A timeout is diagnostic only: a blocked Shell call cannot be safely killed.
    pub fn poll(&mut self) -> Option<DesktopViewWorkerPoll> {
        match self.result_rx.try_recv() {
            Ok(reply) => {
                self.pending_generation = None;
                self.requested_at = None;
                self.timeout_reported = false;
                return Some(match reply.result {
                    Ok(probe) => DesktopViewWorkerPoll::Ready {
                        generation: reply.generation,
                        probe,
                    },
                    Err(message) => DesktopViewWorkerPoll::Failed {
                        generation: reply.generation,
                        message,
                    },
                });
            }
            Err(TryRecvError::Disconnected) => {
                if let Some(generation) = self.pending_generation.take() {
                    self.requested_at = None;
                    return Some(DesktopViewWorkerPoll::Failed {
                        generation,
                        message: "Desktop view worker exited unexpectedly".to_owned(),
                    });
                }
                return None;
            }
            Err(TryRecvError::Empty) => {}
        }

        let generation = self.pending_generation?;
        let requested_at = self.requested_at?;
        if !self.timeout_reported && requested_at.elapsed() >= DESKTOP_VIEW_PROBE_TIMEOUT {
            self.timeout_reported = true;
            return Some(DesktopViewWorkerPoll::TimedOut { generation });
        }
        None
    }

    pub fn has_pending_work(&self) -> bool {
        self.pending_generation.is_some()
    }
}

impl Drop for DesktopViewWorker {
    fn drop(&mut self) {
        // Wake the STA so it can observe channel disconnection.
        self.wake_event.signal();
    }
}
