//! Dedicated BGW-owned out-of-context WinEvent message pump.
//!
//! The first subscription is deliberately limited to a global foreground-change
//! signal. The callback never exports or retains HWND/PID data and performs only
//! an atomic store, which is safe if WinEvent delivery re-enters the callback.
//! The hook is always removed on the thread that installed it.

#[cfg(any(windows, test))]
const EVENT_SYSTEM_FOREGROUND: u32 = 0x0003;
#[cfg(any(windows, test))]
const OBJID_WINDOW: i32 = 0;
#[cfg(any(windows, test))]
const CHILDID_SELF: i32 = 0;

#[cfg(any(windows, test))]
fn is_foreground_window_event(event: u32, has_window: bool, object_id: i32, child_id: i32) -> bool {
    event == EVENT_SYSTEM_FOREGROUND
        && has_window
        && object_id == OBJID_WINDOW
        && child_id == CHILDID_SELF
}

#[cfg(windows)]
mod platform {
    use super::{is_foreground_window_event, EVENT_SYSTEM_FOREGROUND};
    use std::fmt;
    use std::mem::zeroed;
    use std::ptr::{null, null_mut};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc;
    use std::thread::{self, JoinHandle};

    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, HWND};
    use windows_sys::Win32::System::Threading::{CreateEventW, SetEvent};
    use windows_sys::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, MsgWaitForMultipleObjects, PeekMessageW, TranslateMessage, MSG,
    };

    const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;
    const PM_REMOVE: u32 = 0x0001;
    const WM_QUIT: u32 = 0x0012;
    const QS_ALLINPUT: u32 = 0x04FF;
    const WAIT_OBJECT_0: u32 = 0;
    const WAIT_FAILED: u32 = u32::MAX;
    const INFINITE: u32 = u32::MAX;

    static PUMP_ACTIVE: AtomicBool = AtomicBool::new(false);
    static FOREGROUND_CHANGE_PENDING: AtomicBool = AtomicBool::new(false);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(crate) enum NativeEventPumpError {
        AlreadyRunning,
        StopSignalCreation(u32),
        ThreadSpawn,
        HookRegistration(u32),
        StartupChannelClosed,
    }

    impl fmt::Display for NativeEventPumpError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::AlreadyRunning => {
                    formatter.write_str("a BGW WinEvent pump is already active")
                }
                Self::StopSignalCreation(code) => {
                    write!(formatter, "CreateEventW failed with Win32 error {code:#x}")
                }
                Self::ThreadSpawn => {
                    formatter.write_str("the WinEvent pump thread could not start")
                }
                Self::HookRegistration(code) => {
                    write!(
                        formatter,
                        "SetWinEventHook failed with Win32 error {code:#x}"
                    )
                }
                Self::StartupChannelClosed => {
                    formatter.write_str("the WinEvent pump exited before reporting startup")
                }
            }
        }
    }

    /// RAII owner for the foreground WinEvent hook and its message-pump thread.
    pub(crate) struct NativeEventPump {
        stop_signal: HANDLE,
        worker: Option<JoinHandle<()>>,
    }

    impl NativeEventPump {
        pub(crate) fn start() -> Result<Self, NativeEventPumpError> {
            if PUMP_ACTIVE
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
            {
                return Err(NativeEventPumpError::AlreadyRunning);
            }

            FOREGROUND_CHANGE_PENDING.store(false, Ordering::Release);
            let stop_signal = unsafe { CreateEventW(null(), 0, 0, null()) };
            if stop_signal.is_null() {
                PUMP_ACTIVE.store(false, Ordering::Release);
                return Err(NativeEventPumpError::StopSignalCreation(unsafe {
                    GetLastError()
                }));
            }

            let (startup_sender, startup_receiver) = mpsc::sync_channel(1);
            let stop_signal_for_thread = stop_signal as usize;
            let worker = match thread::Builder::new()
                .name("winlook-bgw-winevent".to_owned())
                .spawn(move || pump_thread(stop_signal_for_thread, startup_sender))
            {
                Ok(worker) => worker,
                Err(_) => {
                    unsafe {
                        CloseHandle(stop_signal);
                    }
                    PUMP_ACTIVE.store(false, Ordering::Release);
                    return Err(NativeEventPumpError::ThreadSpawn);
                }
            };

            match startup_receiver.recv() {
                Ok(Ok(())) => Ok(Self {
                    stop_signal,
                    worker: Some(worker),
                }),
                Ok(Err(code)) => {
                    let _ = worker.join();
                    unsafe {
                        CloseHandle(stop_signal);
                    }
                    PUMP_ACTIVE.store(false, Ordering::Release);
                    Err(NativeEventPumpError::HookRegistration(code))
                }
                Err(_) => {
                    unsafe {
                        SetEvent(stop_signal);
                    }
                    let _ = worker.join();
                    unsafe {
                        CloseHandle(stop_signal);
                    }
                    PUMP_ACTIVE.store(false, Ordering::Release);
                    Err(NativeEventPumpError::StartupChannelClosed)
                }
            }
        }

        /// Returns and clears the coalesced foreground-change signal.
        pub(crate) fn take_foreground_changed(&self) -> bool {
            FOREGROUND_CHANGE_PENDING.swap(false, Ordering::AcqRel)
        }
    }

    impl Drop for NativeEventPump {
        fn drop(&mut self) {
            if let Some(worker) = self.worker.take() {
                // The worker exits its wait loop, unhooks on the installing
                // thread, and only then returns to this join.
                unsafe {
                    SetEvent(self.stop_signal);
                }
                let _ = worker.join();
            }
            unsafe {
                CloseHandle(self.stop_signal);
            }
            FOREGROUND_CHANGE_PENDING.store(false, Ordering::Release);
            PUMP_ACTIVE.store(false, Ordering::Release);
        }
    }

    fn pump_thread(stop_signal: usize, startup_sender: mpsc::SyncSender<Result<(), u32>>) {
        let stop_signal = stop_signal as HANDLE;
        let mut message: MSG = unsafe { zeroed() };

        // Force creation of this thread's message queue before the hook is
        // published as ready. MsgWaitForMultipleObjects below services it.
        unsafe {
            PeekMessageW(&mut message, null_mut(), 0, 0, 0);
        }

        let hook = unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                null_mut(),
                Some(on_foreground_win_event),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            )
        };
        if hook.is_null() {
            let code = unsafe { GetLastError() };
            let _ = startup_sender.send(Err(code));
            return;
        }

        if startup_sender.send(Ok(())).is_err() {
            unsafe {
                UnhookWinEvent(hook);
            }
            return;
        }

        let handles = [stop_signal];
        'pump: loop {
            let wait_result = unsafe {
                MsgWaitForMultipleObjects(
                    handles.len() as u32,
                    handles.as_ptr(),
                    0,
                    INFINITE,
                    QS_ALLINPUT,
                )
            };

            if wait_result == WAIT_OBJECT_0 {
                break;
            }
            if wait_result == WAIT_OBJECT_0 + 1 {
                loop {
                    let has_message =
                        unsafe { PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) };
                    if has_message == 0 {
                        break;
                    }
                    if message.message == WM_QUIT {
                        break 'pump;
                    }
                    unsafe {
                        TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                }
                continue;
            }
            if wait_result == WAIT_FAILED {
                break;
            }
            break;
        }

        unsafe {
            UnhookWinEvent(hook);
        }
    }

    unsafe extern "system" fn on_foreground_win_event(
        _hook: HWINEVENTHOOK,
        event: u32,
        hwnd: HWND,
        object_id: i32,
        child_id: i32,
        _event_thread: u32,
        _event_time_ms: u32,
    ) {
        if is_foreground_window_event(event, !hwnd.is_null(), object_id, child_id) {
            // No allocation, lock, COM call, HWND retention or cross-thread wait.
            FOREGROUND_CHANGE_PENDING.store(true, Ordering::Release);
        }
    }
}

#[cfg(windows)]
pub(super) use platform::NativeEventPump;

#[cfg(test)]
mod tests {
    use super::is_foreground_window_event;

    #[test]
    fn accepts_only_top_level_system_foreground_notifications() {
        assert!(is_foreground_window_event(0x0003, true, 0, 0));
        assert!(!is_foreground_window_event(0x0002, true, 0, 0));
        assert!(!is_foreground_window_event(0x0003, false, 0, 0));
        assert!(!is_foreground_window_event(0x0003, true, 1, 0));
        assert!(!is_foreground_window_event(0x0003, true, 0, 1));
    }
}
