//! Message-aware wake event and pump for Windows STA Shell workers.

use std::io;
use std::ptr::{self, NonNull};
use std::sync::Arc;
use std::time::Duration;

type Handle = *mut std::ffi::c_void;

const PM_REMOVE: u32 = 0x0001;
const QS_ALLINPUT: u32 = 0x04ff;
const WAIT_FAILED: u32 = u32::MAX;
const WM_QUIT: u32 = 0x0012;

#[repr(C)]
struct NativePoint {
    x: i32,
    y: i32,
}

#[repr(C)]
struct NativeMessage {
    hwnd: Handle,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    point: NativePoint,
    private: u32,
}

#[link(name = "kernel32")]
extern "system" {
    fn CreateEventW(
        attributes: *const std::ffi::c_void,
        manual_reset: i32,
        initial_state: i32,
        name: *const u16,
    ) -> Handle;
    fn SetEvent(event: Handle) -> i32;
    fn CloseHandle(handle: Handle) -> i32;
}

#[link(name = "user32")]
extern "system" {
    fn PeekMessageW(
        message: *mut NativeMessage,
        window: Handle,
        minimum: u32,
        maximum: u32,
        remove: u32,
    ) -> i32;
    fn TranslateMessage(message: *const NativeMessage) -> i32;
    fn DispatchMessageW(message: *const NativeMessage) -> isize;
    fn MsgWaitForMultipleObjects(
        count: u32,
        handles: *const Handle,
        wait_all: i32,
        milliseconds: u32,
        wake_mask: u32,
    ) -> u32;
}

/// Shared event handle used to signal work and wait without blocking an STA
/// message queue. The Arc keeps the kernel handle valid for both participants.
struct OwnedWakeEvent(NonNull<std::ffi::c_void>);

// SAFETY: Windows event handles are process-wide kernel objects designed for
// cross-thread SetEvent and MsgWaitForMultipleObjects operations.
unsafe impl Send for OwnedWakeEvent {}
unsafe impl Sync for OwnedWakeEvent {}

impl OwnedWakeEvent {
    fn create() -> io::Result<Self> {
        let handle = unsafe { CreateEventW(ptr::null(), 0, 0, ptr::null()) };
        NonNull::new(handle)
            .map(Self)
            .ok_or_else(io::Error::last_os_error)
    }

    fn raw(&self) -> Handle {
        self.0.as_ptr()
    }
}

impl Drop for OwnedWakeEvent {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0.as_ptr());
        }
    }
}

#[derive(Clone)]
pub(crate) struct StaWakeEvent(Arc<OwnedWakeEvent>);

impl StaWakeEvent {
    pub(crate) fn new() -> io::Result<Self> {
        Ok(Self(Arc::new(OwnedWakeEvent::create()?)))
    }

    pub(crate) fn signal(&self) {
        unsafe {
            SetEvent(self.0.raw());
        }
    }

    pub(crate) fn pump_messages(&self) {
        loop {
            let mut message = std::mem::MaybeUninit::<NativeMessage>::uninit();
            let available =
                unsafe { PeekMessageW(message.as_mut_ptr(), ptr::null_mut(), 0, 0, PM_REMOVE) };
            if available == 0 {
                break;
            }
            let message = unsafe { message.assume_init() };
            if message.message == WM_QUIT {
                // Worker lifetime is controlled by its request channel, not by
                // a window message. Do not dispatch WM_QUIT to a window proc.
                continue;
            }
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }

    /// Waits for the request event or queued STA input, then dispatches messages.
    pub(crate) fn wait_and_pump(&self, timeout: Duration) -> io::Result<()> {
        let handles = [self.0.raw()];
        let milliseconds = timeout.as_millis().min(u128::from(u32::MAX)) as u32;
        let result = unsafe {
            MsgWaitForMultipleObjects(
                handles.len() as u32,
                handles.as_ptr(),
                0,
                milliseconds,
                QS_ALLINPUT,
            )
        };
        if result == WAIT_FAILED {
            return Err(io::Error::last_os_error());
        }
        self.pump_messages();
        Ok(())
    }
}
