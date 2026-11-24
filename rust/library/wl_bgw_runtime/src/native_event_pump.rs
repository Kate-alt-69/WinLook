//! Dedicated BGW-owned out-of-context WinEvent message pump.
//!
//! BGW owns hook-thread lifecycle, message dispatch, wake signaling and
//! same-thread teardown. Traton owns the pure WinEvent category classifier;
//! this callback only records categories in an atomic mask.

#[cfg(windows)]
mod platform {
    use std::fmt;
    use std::mem::zeroed;
    use std::ptr::{null, null_mut};
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::mpsc;
    use std::thread::{self, JoinHandle};

    use wl_traton::{classify_win_event, WIN_EVENT_HOOK_RANGES};

    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, HWND};
    use windows_sys::Win32::System::Threading::{CreateEventW, SetEvent};
    use windows_sys::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, MsgWaitForMultipleObjects, PeekMessageW, TranslateMessage, MSG,
    };
    use wl_wlde_win32::{DesktopInventoryError, DesktopInventoryWakeSignal};

    const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;
    const WINEVENT_SKIPOWNPROCESS: u32 = 0x0002;
    const PM_REMOVE: u32 = 0x0001;
    const WM_QUIT: u32 = 0x0012;
    const QS_ALLINPUT: u32 = 0x04FF;
    const WAIT_OBJECT_0: u32 = 0;
    const WAIT_FAILED: u32 = u32::MAX;
    const INFINITE: u32 = u32::MAX;

    static PUMP_ACTIVE: AtomicBool = AtomicBool::new(false);
    static PUMP_FAILED: AtomicBool = AtomicBool::new(false);
    static PUMP_FAILURE_CODE: AtomicU32 = AtomicU32::new(0);
    static PENDING_EVENTS: AtomicU32 = AtomicU32::new(0);
    static EVENT_WAKE_PENDING: AtomicBool = AtomicBool::new(false);

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
                        "one of the BGW SetWinEventHook subscriptions failed with Win32 error {code:#x}"
                    )
                }
                Self::StartupChannelClosed => {
                    formatter.write_str("the WinEvent pump exited before reporting startup")
                }
            }
        }
    }

    /// RAII owner for the window-event hooks and their message-pump thread.
    pub(crate) struct NativeEventPump {
        stop_signal: HANDLE,
        worker: Option<JoinHandle<()>>,
    }

    impl NativeEventPump {
        pub(crate) fn start(
            wake_signal: Option<DesktopInventoryWakeSignal>,
        ) -> Result<Self, NativeEventPumpError> {
            if PUMP_ACTIVE
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
            {
                return Err(NativeEventPumpError::AlreadyRunning);
            }

            PUMP_FAILED.store(false, Ordering::Release);
            PUMP_FAILURE_CODE.store(0, Ordering::Release);
            PENDING_EVENTS.store(0, Ordering::Release);
            EVENT_WAKE_PENDING.store(false, Ordering::Release);
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
                .spawn(move || pump_thread(stop_signal_for_thread, wake_signal, startup_sender))
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

        /// Returns and clears event categories coalesced since the last poll.
        pub(crate) fn take_pending_events(&self) -> u32 {
            PENDING_EVENTS.swap(0, Ordering::AcqRel)
        }

        /// Whether the native message-pump thread is still alive.
        pub(crate) fn is_running(&self) -> bool {
            !PUMP_FAILED.load(Ordering::Acquire)
                && self
                    .worker
                    .as_ref()
                    .is_some_and(|worker| !worker.is_finished())
        }

        /// Error or wait result recorded when the pump exits unexpectedly.
        pub(crate) fn failure_code(&self) -> Option<u32> {
            PUMP_FAILED
                .load(Ordering::Acquire)
                .then(|| PUMP_FAILURE_CODE.load(Ordering::Acquire))
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
            PENDING_EVENTS.store(0, Ordering::Release);
            EVENT_WAKE_PENDING.store(false, Ordering::Release);
            PUMP_FAILED.store(false, Ordering::Release);
            PUMP_FAILURE_CODE.store(0, Ordering::Release);
            PUMP_ACTIVE.store(false, Ordering::Release);
        }
    }

    fn pump_thread(
        stop_signal: usize,
        wake_signal: Option<DesktopInventoryWakeSignal>,
        startup_sender: mpsc::SyncSender<Result<(), u32>>,
    ) {
        let stop_signal = stop_signal as HANDLE;
        let mut message: MSG = unsafe { zeroed() };

        // Force creation of this thread's message queue before the hooks are
        // published as ready. MsgWaitForMultipleObjects below services it.
        unsafe {
            PeekMessageW(&mut message, null_mut(), 0, 0, 0);
        }

        // Traton is the single source of truth for the selected event ranges.
        let mut hooks: Vec<HWINEVENTHOOK> = Vec::with_capacity(WIN_EVENT_HOOK_RANGES.len());
        for &(event_min, event_max) in WIN_EVENT_HOOK_RANGES {
            let hook = unsafe {
                SetWinEventHook(
                    event_min,
                    event_max,
                    null_mut(),
                    Some(on_window_win_event),
                    0,
                    0,
                    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                )
            };
            if hook.is_null() {
                let code = unsafe { GetLastError() };
                for registered in hooks {
                    unsafe {
                        UnhookWinEvent(registered);
                    }
                }
                let _ = startup_sender.send(Err(code));
                return;
            }
            hooks.push(hook);
        }

        if startup_sender.send(Ok(())).is_err() {
            for hook in hooks {
                unsafe {
                    UnhookWinEvent(hook);
                }
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
                if EVENT_WAKE_PENDING.swap(false, Ordering::AcqRel) {
                    if let Some(wake_signal) = wake_signal.as_ref() {
                        if let Err(error) = wake_signal.signal() {
                            let code = match error {
                                DesktopInventoryError::SignalWake { code } => code,
                                _ => 0,
                            };
                            PUMP_FAILURE_CODE.store(code, Ordering::Release);
                            PUMP_FAILED.store(true, Ordering::Release);
                            break;
                        }
                    }
                }
                continue;
            }

            let failure_code = if wait_result == WAIT_FAILED {
                unsafe { GetLastError() }
            } else {
                wait_result
            };
            PUMP_FAILURE_CODE.store(failure_code, Ordering::Release);
            PUMP_FAILED.store(true, Ordering::Release);
            if let Some(wake_signal) = wake_signal.as_ref() {
                let _ = wake_signal.signal();
            }
            break;
        }

        // Unhook on the same thread that installed each subscription.
        for hook in hooks {
            unsafe {
                UnhookWinEvent(hook);
            }
        }
    }

    unsafe extern "system" fn on_window_win_event(
        _hook: HWINEVENTHOOK,
        event: u32,
        hwnd: HWND,
        object_id: i32,
        child_id: i32,
        _event_thread: u32,
        _event_time_ms: u32,
    ) {
        if let Some(category) = classify_win_event(event, !hwnd.is_null(), object_id, child_id) {
            // No allocation, lock, COM call, HWND retention or cross-thread wait.
            PENDING_EVENTS.fetch_or(category.bit(), Ordering::AcqRel);
            EVENT_WAKE_PENDING.store(true, Ordering::Release);
        }
    }
}

#[cfg(windows)]
pub(super) use platform::NativeEventPump;
