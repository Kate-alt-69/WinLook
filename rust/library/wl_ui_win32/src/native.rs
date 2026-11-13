use std::ffi::c_void;
use wl_ui_core::{UiApplication, UiError, UiPage};

type Handle = *mut c_void;
type WindowProcedure = unsafe extern "system" fn(Handle, u32, usize, isize) -> isize;

const WM_NCCREATE: u32 = 0x0081;
const WM_CREATE: u32 = 0x0001;
const WM_SIZE: u32 = 0x0005;
const WM_DESTROY: u32 = 0x0002;
const WM_COMMAND: u32 = 0x0111;
const WM_NCDESTROY: u32 = 0x0082;
const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
const WS_CHILD: u32 = 0x40000000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_VSCROLL: u32 = 0x00200000;
const WS_TABSTOP: u32 = 0x00010000;
const WS_BORDER: u32 = 0x00800000;
const ES_MULTILINE: u32 = 0x0004;
const ES_AUTOVSCROLL: u32 = 0x0040;
const ES_READONLY: u32 = 0x0800;
const LBS_NOTIFY: u32 = 0x0001;
const LBS_NOINTEGRALHEIGHT: u32 = 0x0100;
const LB_ADDSTRING: u32 = 0x0180;
const LB_GETCURSEL: u32 = 0x0188;
const LB_SETCURSEL: u32 = 0x0186;
const LBN_SELCHANGE: usize = 1;
const SW_SHOW: i32 = 5;
const CW_USEDEFAULT: i32 = 0x80000000u32 as i32;
const ERROR_CLASS_ALREADY_EXISTS: u32 = 1410;
const NAVIGATION_CONTROL_ID: usize = 1001;
const CONTENT_CONTROL_ID: usize = 1002;
const CLASS_NAME: &str = "WinLook.WlUi.Win32";
const WINDOW_STATE_PROPERTY: &str = "WinLook.WlUi.State";

#[repr(C)]
struct Point {
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
    point: Point,
    private: u32,
}

#[repr(C)]
struct WindowClass {
    style: u32,
    window_proc: Option<WindowProcedure>,
    class_extra: i32,
    window_extra: i32,
    instance: Handle,
    icon: Handle,
    cursor: Handle,
    background: Handle,
    menu_name: *const u16,
    class_name: *const u16,
}

#[repr(C)]
struct CreateStruct {
    create_parameters: *mut c_void,
    instance: Handle,
    menu: Handle,
    parent: Handle,
    height: i32,
    width: i32,
    y: i32,
    x: i32,
    style: i32,
    name: *const u16,
    class_name: *const u16,
    extended_style: u32,
}

#[repr(C)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

struct WindowState {
    application: UiApplication,
    navigation: Handle,
    content: Handle,
}

#[link(name = "user32")]
extern "system" {
    fn RegisterClassW(class: *const WindowClass) -> u16;
    fn CreateWindowExW(
        extended_style: u32,
        class_name: *const u16,
        window_name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Handle,
        menu: Handle,
        instance: Handle,
        parameter: *mut c_void,
    ) -> Handle;
    fn DefWindowProcW(hwnd: Handle, message: u32, w_param: usize, l_param: isize) -> isize;
    fn ShowWindow(hwnd: Handle, command: i32) -> i32;
    fn UpdateWindow(hwnd: Handle) -> i32;
    fn GetMessageW(message: *mut NativeMessage, hwnd: Handle, min: u32, max: u32) -> i32;
    fn TranslateMessage(message: *const NativeMessage) -> i32;
    fn DispatchMessageW(message: *const NativeMessage) -> isize;
    fn PostQuitMessage(exit_code: i32);
    fn LoadCursorW(instance: Handle, name: *const u16) -> Handle;
    fn SetPropW(hwnd: Handle, name: *const u16, data: Handle) -> i32;
    fn GetPropW(hwnd: Handle, name: *const u16) -> Handle;
    fn RemovePropW(hwnd: Handle, name: *const u16) -> Handle;
    fn SendMessageW(hwnd: Handle, message: u32, w_param: usize, l_param: isize) -> isize;
    fn SetWindowTextW(hwnd: Handle, text: *const u16) -> i32;
    fn GetClientRect(hwnd: Handle, rect: *mut Rect) -> i32;
    fn MoveWindow(hwnd: Handle, x: i32, y: i32, width: i32, height: i32, repaint: i32) -> i32;
    fn MessageBoxW(hwnd: Handle, text: *const u16, caption: *const u16, kind: u32) -> i32;
    fn DestroyWindow(hwnd: Handle) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Handle;
    fn GetLastError() -> u32;
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn property_name() -> Vec<u16> {
    wide(WINDOW_STATE_PROPERTY)
}

fn state_for(hwnd: Handle) -> *mut WindowState {
    let name = property_name();
    unsafe { GetPropW(hwnd, name.as_ptr()).cast() }
}

unsafe extern "system" fn window_proc(
    hwnd: Handle,
    message: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    if message == WM_NCCREATE {
        let create = unsafe { &*(l_param as *const CreateStruct) };
        let name = property_name();
        let stored = unsafe {
            SetPropW(hwnd, name.as_ptr(), create.create_parameters)
        };
        return if stored != 0 { 1 } else { 0 };
    }

    let state = state_for(hwnd);

    match message {
        WM_CREATE => {
            if state.is_null() {
                return -1;
            }

            let application = unsafe { &(*state).application };
            let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
            let list_class = wide("LISTBOX");
            let content_class = wide("EDIT");
            let navigation = unsafe {
                CreateWindowExW(
                    0,
                    list_class.as_ptr(),
                    std::ptr::null(),
                    WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_TABSTOP
                        | WS_BORDER | LBS_NOTIFY | LBS_NOINTEGRALHEIGHT,
                    16,
                    64,
                    248,
                    600,
                    hwnd,
                    NAVIGATION_CONTROL_ID as Handle,
                    instance,
                    std::ptr::null_mut(),
                )
            };
            let content = unsafe {
                CreateWindowExW(
                    0,
                    content_class.as_ptr(),
                    std::ptr::null(),
                    WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_BORDER
                        | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY,
                    280,
                    64,
                    900,
                    600,
                    hwnd,
                    CONTENT_CONTROL_ID as Handle,
                    instance,
                    std::ptr::null_mut(),
                )
            };

            if navigation.is_null() || content.is_null() {
                return -1;
            }

            unsafe {
                (*state).navigation = navigation;
                (*state).content = content;
            }

            for page in application.pages() {
                let title = wide(page.title());
                unsafe {
                    SendMessageW(navigation, LB_ADDSTRING, 0, title.as_ptr() as isize);
                }
            }

            let initial_index = application
                .pages()
                .iter()
                .position(|page| page.id() == application.initial_page_id())
                .unwrap_or(0);

            unsafe {
                SendMessageW(navigation, LB_SETCURSEL, initial_index, 0);
                set_page_content(content, &application.pages()[initial_index]);
            }

            0
        }
        WM_SIZE => {
            if !state.is_null() {
                let mut rect = Rect {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                if unsafe { GetClientRect(hwnd, &mut rect) } != 0 {
                    let width = rect.right - rect.left;
                    let height = rect.bottom - rect.top;
                    let navigation_width = 248.min(width.saturating_sub(24));
                    unsafe {
                        MoveWindow(
                            (*state).navigation,
                            16,
                            64,
                            navigation_width,
                            height.saturating_sub(80),
                            1,
                        );
                        MoveWindow(
                            (*state).content,
                            navigation_width + 32,
                            64,
                            width.saturating_sub(navigation_width + 48),
                            height.saturating_sub(80),
                            1,
                        );
                    }
                }
            }
            0
        }
        WM_COMMAND => {
            let control_id = w_param & 0xFFFF;
            let notification = (w_param >> 16) & 0xFFFF;
            if !state.is_null()
                && control_id == NAVIGATION_CONTROL_ID
                && notification == LBN_SELCHANGE
            {
                let application = unsafe { &(*state).application };
                let selected = unsafe {
                    SendMessageW((*state).navigation, LB_GETCURSEL, 0, 0)
                };
                if selected >= 0 {
                    if let Some(page) = application.pages().get(selected as usize) {
                        unsafe { set_page_content((*state).content, page) };
                    }
                }
            }
            0
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }
        WM_NCDESTROY => {
            let name = property_name();
            unsafe {
                RemovePropW(hwnd, name.as_ptr());
                DefWindowProcW(hwnd, message, w_param, l_param)
            }
        }
        _ => unsafe { DefWindowProcW(hwnd, message, w_param, l_param) },
    }
}

unsafe fn set_page_content(hwnd: Handle, page: &UiPage) {
    let body = format!("{}\r\n\r\n{}", page.title(), page.description());
    let text = wide(&body);
    unsafe {
        SetWindowTextW(hwnd, text.as_ptr());
    }
}

fn show_start_error(error: &UiError, title: &str) {
    let message = wide(&error.to_string());
    let caption = wide(title);
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            message.as_ptr(),
            caption.as_ptr(),
            0x00000010,
        );
    }
}

fn register_window_class(instance: Handle) -> Result<(), UiError> {
    let class_name = wide(CLASS_NAME);
    let cursor = unsafe {
        LoadCursorW(std::ptr::null_mut(), 32512usize as *const u16)
    };
    let class = WindowClass {
        style: 0x0002 | 0x0001,
        window_proc: Some(window_proc),
        class_extra: 0,
        window_extra: 0,
        instance,
        icon: std::ptr::null_mut(),
        cursor,
        background: 6usize as Handle,
        menu_name: std::ptr::null(),
        class_name: class_name.as_ptr(),
    };

    let registered = unsafe { RegisterClassW(&class) };
    if registered == 0 && unsafe { GetLastError() } != ERROR_CLASS_ALREADY_EXISTS {
        return Err(UiError::Backend(
            "RegisterClassW failed for the native Win32 UI".to_string(),
        ));
    }
    Ok(())
}

pub fn run(application: UiApplication) -> Result<(), UiError> {
    let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
    register_window_class(instance)?;

    let state = Box::new(WindowState {
        application: application.clone(),
        navigation: std::ptr::null_mut(),
        content: std::ptr::null_mut(),
    });
    let state_pointer = Box::into_raw(state);
    let class_name = wide(CLASS_NAME);
    let title = wide(application.window_config().title());

    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            application.window_config().width() as i32,
            application.window_config().height() as i32,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance,
            state_pointer.cast(),
        )
    };

    if hwnd.is_null() {
        unsafe { drop(Box::from_raw(state_pointer)) };
        let error = UiError::Backend(
            "CreateWindowExW failed for the native Win32 UI".to_string(),
        );
        show_start_error(&error, application.window_config().title());
        return Err(error);
    }

    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }

    let mut message = NativeMessage {
        hwnd: std::ptr::null_mut(),
        message: 0,
        w_param: 0,
        l_param: 0,
        time: 0,
        point: Point { x: 0, y: 0 },
        private: 0,
    };

    let mut loop_error = false;
    loop {
        let result = unsafe {
            GetMessageW(&mut message, std::ptr::null_mut(), 0, 0)
        };
        if result == -1 {
            loop_error = true;
            break;
        }
        if result == 0 {
            break;
        }
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    if loop_error {
        unsafe { DestroyWindow(hwnd) };
    }
    unsafe { drop(Box::from_raw(state_pointer)) };

    if loop_error {
        let error = UiError::Backend("GetMessageW failed in the native Win32 message loop".to_string());
        show_start_error(&error, application.window_config().title());
        Err(error)
    } else {
        Ok(())
    }
}
