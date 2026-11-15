use crate::{enumerate_monitors, should_refresh_topology};
use std::ffi::c_void;
use std::mem::size_of;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use wl_ui_core::{UiApplication, UiColor, UiComponent, UiError, UiPage, UiPalette};
use wl_wlde::{store_provider, StoreProviderId, WallpaperEngineInstall, WallpaperTarget};

type Handle = *mut c_void;
type WindowProcedure = unsafe extern "system" fn(Handle, u32, usize, isize) -> isize;

const WM_NCCREATE: u32 = 0x0081;
const WM_CREATE: u32 = 0x0001;
const WM_SIZE: u32 = 0x0005;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_SETFONT: u32 = 0x0030;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_CTLCOLORLISTBOX: u32 = 0x0134;
const WM_CTLCOLORBTN: u32 = 0x0135;
const WM_CTLCOLORSTATIC: u32 = 0x0138;
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
const SW_HIDE: i32 = 0;
const OPAQUE: i32 = 2;
const CW_USEDEFAULT: i32 = 0x80000000u32 as i32;
const ERROR_CLASS_ALREADY_EXISTS: u32 = 1410;
const NAVIGATION_CONTROL_ID: usize = 1001;
const CONTENT_CONTROL_ID: usize = 1002;
const BRAND_MARK_CONTROL_ID: usize = 1003;
const BRAND_TITLE_CONTROL_ID: usize = 1004;
const BRAND_TAGLINE_CONTROL_ID: usize = 1005;
const PAGE_TITLE_CONTROL_ID: usize = 1006;
const PAGE_DESCRIPTION_CONTROL_ID: usize = 1007;
const SS_CENTER: u32 = 0x00000001;
const LOCATE_ENGINE_BUTTON_ID: usize = 1101;
const BROWSE_WALLPAPER_BUTTON_ID: usize = 1102;
const WORKSHOP_BUTTON_ID: usize = 1103;
const OPENVERSE_BUTTON_ID: usize = 1104;
const COMMONS_BUTTON_ID: usize = 1105;
const NASA_BUTTON_ID: usize = 1106;
const REFRESH_DISPLAYS_BUTTON_ID: usize = 1107;
const BUTTON_COUNT: usize = 7;
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
    brand_mark: Handle,
    brand_title: Handle,
    brand_tagline: Handle,
    page_title: Handle,
    page_description: Handle,
    window_brush: Handle,
    navigation_brush: Handle,
    content_brush: Handle,
    brand_brush: Handle,
    font: Handle,
    brand_font: Handle,
    heading_font: Handle,
    navigation_color: UiColor,
    content_color: UiColor,
    brand_color: UiColor,
    brand_text_color: UiColor,
    text_color: UiColor,
    secondary_text_color: UiColor,
    action_buttons: [Handle; BUTTON_COUNT],
    wallpaper_engine: Option<WallpaperEngineInstall>,
    display_status: String,
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
    fn FillRect(dc: Handle, rect: *const Rect, brush: Handle) -> i32;

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

#[repr(C)]
struct OpenFileNameW {
    l_struct_size: u32,
    hwnd_owner: Handle,
    h_instance: Handle,
    filter: *const u16,
    custom_filter: *mut u16,
    max_custom_filter: u32,
    filter_index: u32,
    file: *mut u16,
    max_file: u32,
    file_title: *mut u16,
    max_file_title: u32,
    initial_dir: *const u16,
    title: *const u16,
    flags: u32,
    file_offset: u16,
    file_extension: u16,
    default_extension: *const u16,
    custom_data: isize,
    hook: Option<unsafe extern "system" fn(Handle, u32, usize, isize) -> u32>,
    template_name: *const u16,
    reserved: *mut c_void,
    reserved_word: u32,
    flags_ex: u32,
}

#[link(name = "comdlg32")]
extern "system" {
    fn GetOpenFileNameW(dialog: *mut OpenFileNameW) -> i32;
    fn CommDlgExtendedError() -> u32;
}

#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteW(
        hwnd: Handle,
        operation: *const u16,
        file: *const u16,
        parameters: *const u16,
        directory: *const u16,
        show_command: i32,
    ) -> Handle;
}

const OFN_FILEMUSTEXIST: u32 = 0x00001000;
const OFN_PATHMUSTEXIST: u32 = 0x00000800;
const OFN_NOCHANGEDIR: u32 = 0x00000008;
const OFN_EXPLORER: u32 = 0x00080000;

#[link(name = "gdi32")]
extern "system" {
    fn CreateSolidBrush(color: u32) -> Handle;
    fn DeleteObject(object: Handle) -> i32;
    fn SetTextColor(dc: Handle, color: u32) -> u32;
    fn SetBkColor(dc: Handle, color: u32) -> u32;
    fn SetBkMode(dc: Handle, mode: i32) -> i32;
    fn CreateFontW(
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strike_out: u32,
        character_set: u32,
        output_precision: u32,
        clip_precision: u32,
        quality: u32,
        pitch_and_family: u32,
        face_name: *const u16,
    ) -> Handle;
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn browse_file(
    title: &str,
    filter_description: &str,
    filter_pattern: &str,
) -> Result<Option<PathBuf>, UiError> {
    browse_file_from(title, filter_description, filter_pattern, None)
}

pub(super) fn browse_file_from(
    title: &str,
    filter_description: &str,
    filter_pattern: &str,
    initial_directory: Option<&std::path::Path>,
) -> Result<Option<PathBuf>, UiError> {
    let mut filter = Vec::new();
    filter.extend(filter_description.encode_utf16());
    filter.push(0);
    filter.extend(filter_pattern.encode_utf16());
    filter.push(0);
    filter.extend("All files".encode_utf16());
    filter.push(0);
    filter.extend("*.*".encode_utf16());
    filter.push(0);
    filter.push(0);

    let title = wide(title);
    let initial_directory = initial_directory.map(|path| {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>()
    });
    let initial_dir_ptr = initial_directory
        .as_ref()
        .map_or(std::ptr::null(), |value| value.as_ptr());
    let mut file = vec![0u16; 32_768];
    let mut dialog = OpenFileNameW {
        l_struct_size: size_of::<OpenFileNameW>() as u32,
        hwnd_owner: std::ptr::null_mut(),
        h_instance: std::ptr::null_mut(),
        filter: filter.as_ptr(),
        custom_filter: std::ptr::null_mut(),
        max_custom_filter: 0,
        filter_index: 1,
        file: file.as_mut_ptr(),
        max_file: file.len() as u32,
        file_title: std::ptr::null_mut(),
        max_file_title: 0,
        initial_dir: initial_dir_ptr,
        title: title.as_ptr(),
        flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR | OFN_EXPLORER,
        file_offset: 0,
        file_extension: 0,
        default_extension: std::ptr::null(),
        custom_data: 0,
        hook: None,
        template_name: std::ptr::null(),
        reserved: std::ptr::null_mut(),
        reserved_word: 0,
        flags_ex: 0,
    };

    if unsafe { GetOpenFileNameW(&mut dialog) } == 0 {
        let error = unsafe { CommDlgExtendedError() };
        if error == 0 {
            return Ok(None);
        }
        return Err(UiError::Backend(format!(
            "Windows file picker failed: {error:#x}"
        )));
    }
    let length = file
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(file.len());
    if length == 0 {
        return Ok(None);
    }
    Ok(Some(PathBuf::from(std::ffi::OsString::from_wide(
        &file[..length],
    ))))
}

pub fn open_url(url: &str) -> Result<(), UiError> {
    if !url.starts_with("https://") || url.contains('\0') {
        return Err(UiError::Backend("only HTTPS links are allowed".to_string()));
    }
    let operation = wide("open");
    let target = wide(url);
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            target.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOW,
        )
    };
    if (result as isize) <= 32 {
        return Err(UiError::Backend(format!(
            "Windows could not open the browser link: {}",
            result as isize
        )));
    }
    Ok(())
}

fn monitor_snapshot_status() -> String {
    match enumerate_monitors() {
        Ok(topology) => {
            let primary = topology
                .primary()
                .map(|monitor| monitor.id().to_string())
                .unwrap_or_else(|| "not reported".to_string());
            format!(
                "{} attached display(s) in the latest snapshot. Primary: {}. The Win32 Manager refreshes this snapshot on display/device notifications.",
                topology.len(),
                primary
            )
        }
        Err(error) => format!("Display snapshot unavailable: {error}"),
    }
}

fn colorref(color: UiColor) -> u32 {
    u32::from(color.red) | (u32::from(color.green) << 8) | (u32::from(color.blue) << 16)
}

fn create_static_label(
    parent: Handle,
    instance: Handle,
    id: usize,
    label: &str,
    extra_style: u32,
) -> Handle {
    let class_name = wide("STATIC");
    let title = wide(label);
    unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_CHILD | WS_VISIBLE | extra_style,
            0,
            0,
            160,
            28,
            parent,
            id as Handle,
            instance,
            std::ptr::null_mut(),
        )
    }
}

fn create_action_button(parent: Handle, instance: Handle, id: usize, label: &str) -> Handle {
    let class_name = wide("BUTTON");
    let title = wide(label);
    unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_CHILD | WS_TABSTOP,
            0,
            16,
            160,
            32,
            parent,
            id as Handle,
            instance,
            std::ptr::null_mut(),
        )
    }
}

fn update_action_buttons(state: &WindowState, page: &UiPage) {
    let visible = match page.id() {
        "wallpaper-store-wallpaper-engine" => [true, true, true, false, false, false, false],
        "wallpaper-store-free" => [false, false, false, true, true, true, false],
        "displays" => [false, false, false, false, false, false, true],
        _ => [false, false, false, false, false, false, false],
    };
    for (index, button) in state.action_buttons.iter().enumerate() {
        if !button.is_null() {
            unsafe {
                ShowWindow(*button, if visible[index] { SW_SHOW } else { SW_HIDE });
            }
        }
    }
}

fn refresh_current_page(state: &WindowState) {
    let selected = unsafe { SendMessageW(state.navigation, LB_GETCURSEL, 0, 0) };
    if selected >= 0 {
        if let Some(page) = state.application.pages().get(selected as usize) {
            unsafe {
                set_page_content(state, page);
            }
            update_action_buttons(state, page);
        }
    }
}

fn show_action_error(error: &UiError) {
    show_start_error(error, "WinLook Manager");
}

fn show_action_message(message: &str) {
    let text = wide(message);
    let title = wide("WinLook Manager");
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            0x00000040,
        );
    }
}

fn handle_store_command(state_pointer: *mut WindowState, control_id: usize) {
    if state_pointer.is_null() {
        return;
    }
    let state = unsafe { &mut *state_pointer };
    match control_id {
        LOCATE_ENGINE_BUTTON_ID => match browse_file(
            "Locate Wallpaper Engine",
            "Wallpaper Engine executable",
            "wallpaper64.exe;wallpaper32.exe",
        ) {
            Ok(Some(path)) => match WallpaperEngineInstall::from_executable(path) {
                Ok(install) => {
                    let workshop = install
                        .workshop_directory()
                        .filter(|path| path.is_dir())
                        .map(|path| path.display().to_string());
                    state.wallpaper_engine = Some(install);
                    show_action_message(&workshop.map_or_else(
                        || "Wallpaper Engine was located. Browse to a local project.json or supported media file.".to_string(),
                        |path| format!("Wallpaper Engine was located. Workshop wallpapers are available at {path}."),
                    ));
                }
                Err(error) => show_action_error(&UiError::Backend(error.to_string())),
            },
            Ok(None) => {}
            Err(error) => show_action_error(&error),
        },
        BROWSE_WALLPAPER_BUTTON_ID => {
            let Some(install) = state.wallpaper_engine.as_ref() else {
                show_action_error(&UiError::Backend(
                    "Wallpaper Engine was not found. Use Locate Wallpaper Engine first."
                        .to_string(),
                ));
                refresh_current_page(state);
                return;
            };
            let workshop_directory = install.workshop_directory().filter(|path| path.is_dir());
            match browse_file_from(
                "Browse Wallpaper Engine wallpapers",
                "Wallpaper Engine project or media",
                "project.json;*.mp4;*.webm;*.avi;*.mov;*.m4v;*.wmv;*.gif;*.png;*.jpg;*.jpeg;*.webp;*.bmp",
                workshop_directory.as_deref(),
            ) {
                Ok(Some(path)) => match WallpaperTarget::new(path) {
                    Ok(target) => match install.open_wallpaper(&target, None) {
                        Ok(_child) => show_action_message(
                            "The openWallpaper command was sent. Wallpaper Engine should already be running; Steam still manages Workshop subscriptions and downloads.",
                        ),
                        Err(error) => show_action_error(&UiError::Backend(error.to_string())),
                    },
                    Err(error) => show_action_error(&UiError::Backend(error.to_string())),
                },
                Ok(None) => {}
                Err(error) => show_action_error(&error),
            }
        }
        WORKSHOP_BUTTON_ID => {
            if let Err(error) =
                open_url(store_provider(StoreProviderId::WallpaperEngine).browse_url)
            {
                show_action_error(&error);
            }
        }
        OPENVERSE_BUTTON_ID => {
            if let Err(error) = open_url(store_provider(StoreProviderId::Openverse).browse_url) {
                show_action_error(&error);
            }
        }
        COMMONS_BUTTON_ID => {
            if let Err(error) =
                open_url(store_provider(StoreProviderId::WikimediaCommons).browse_url)
            {
                show_action_error(&error);
            }
        }
        NASA_BUTTON_ID => {
            if let Err(error) = open_url(store_provider(StoreProviderId::NasaImages).browse_url) {
                show_action_error(&error);
            }
        }
        REFRESH_DISPLAYS_BUTTON_ID => {
            state.display_status = monitor_snapshot_status();
            show_action_message(&state.display_status);
        }
        _ => {}
    }
    refresh_current_page(state);
}

fn component_background(application: &UiApplication, component: UiComponent) -> Option<UiColor> {
    application
        .styles_ref()
        .rules()
        .iter()
        .rev()
        .find(|rule| rule.selector().component() == &component)
        .and_then(|rule| rule.style().background_color())
}

fn component_foreground(application: &UiApplication, component: UiComponent) -> Option<UiColor> {
    application
        .styles_ref()
        .rules()
        .iter()
        .rev()
        .find(|rule| rule.selector().component() == &component)
        .and_then(|rule| rule.style().foreground_color())
}

fn create_font_at(application: &UiApplication, size: f32, weight: u16) -> Handle {
    let typography = application.theme_ref().typography_tokens();
    let family = match typography.font_family.as_str() {
        "system-ui" | "sans-serif" => "Segoe UI",
        "monospace" => "Consolas",
        value => value,
    };
    let face = wide(family);
    unsafe {
        CreateFontW(
            -(size.round() as i32),
            0,
            0,
            0,
            weight as i32,
            0,
            0,
            0,
            1,
            0,
            0,
            5,
            0,
            face.as_ptr(),
        )
    }
}

fn create_font(application: &UiApplication) -> Handle {
    let typography = application.theme_ref().typography_tokens();
    create_font_at(application, typography.size_md, typography.weight_regular)
}

fn delete_state_resources(state: &WindowState) {
    unsafe {
        if !state.window_brush.is_null() {
            DeleteObject(state.window_brush);
        }
        if !state.navigation_brush.is_null() {
            DeleteObject(state.navigation_brush);
        }
        if !state.content_brush.is_null() {
            DeleteObject(state.content_brush);
        }
        if !state.brand_brush.is_null() {
            DeleteObject(state.brand_brush);
        }
        for font in [state.font, state.brand_font, state.heading_font] {
            if !font.is_null() {
                DeleteObject(font);
            }
        }
    }
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
        let stored = unsafe { SetPropW(hwnd, name.as_ptr(), create.create_parameters) };
        return if stored != 0 { 1 } else { 0 };
    }

    let state = state_for(hwnd);

    if !state.is_null()
        && unsafe { !(*state).navigation.is_null() && !(*state).content.is_null() }
        && should_refresh_topology(message, w_param)
    {
        unsafe {
            (*state).display_status = monitor_snapshot_status();
            refresh_current_page(&*state);
        }
    }

    if message == WM_ERASEBKGND && !state.is_null() {
        let mut rect = Rect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if unsafe { GetClientRect(hwnd, &mut rect) } != 0 {
            unsafe {
                FillRect(w_param as Handle, &rect, (*state).window_brush);
            }
        }
        return 1;
    }

    if !state.is_null()
        && (message == WM_CTLCOLOREDIT
            || message == WM_CTLCOLORLISTBOX
            || message == WM_CTLCOLORSTATIC
            || message == WM_CTLCOLORBTN)
    {
        let control = l_param as Handle;
        let (brush, background, foreground) = unsafe {
            match message {
                WM_CTLCOLORLISTBOX => (
                    (*state).navigation_brush,
                    (*state).navigation_color,
                    (*state).text_color,
                ),
                WM_CTLCOLOREDIT => (
                    (*state).content_brush,
                    (*state).content_color,
                    (*state).text_color,
                ),
                WM_CTLCOLORSTATIC if control == (*state).brand_mark => (
                    (*state).brand_brush,
                    (*state).brand_color,
                    (*state).brand_text_color,
                ),
                WM_CTLCOLORSTATIC
                    if control == (*state).brand_tagline
                        || control == (*state).page_description => (
                    (*state).content_brush,
                    (*state).content_color,
                    (*state).secondary_text_color,
                ),
                WM_CTLCOLORSTATIC => (
                    (*state).content_brush,
                    (*state).content_color,
                    (*state).text_color,
                ),
                _ => (
                    (*state).navigation_brush,
                    (*state).navigation_color,
                    (*state).text_color,
                ),
            }
        };
        let dc = w_param as Handle;
        unsafe {
            SetTextColor(dc, colorref(foreground));
            SetBkColor(dc, colorref(background));
            SetBkMode(dc, OPAQUE);
        }
        return brush as isize;
    }

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
                    WS_CHILD
                        | WS_VISIBLE
                        | WS_VSCROLL
                        | WS_TABSTOP
                        | WS_BORDER
                        | LBS_NOTIFY
                        | LBS_NOINTEGRALHEIGHT,
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
                    WS_CHILD
                        | WS_VISIBLE
                        | WS_VSCROLL
                        | WS_BORDER
                        | ES_MULTILINE
                        | ES_AUTOVSCROLL
                        | ES_READONLY,
                    280,
                    144,
                    900,
                    520,
                    hwnd,
                    CONTENT_CONTROL_ID as Handle,
                    instance,
                    std::ptr::null_mut(),
                )
            };
            let branding = application.branding_ref();
            let brand_mark_text = branding
                .map(|brand| brand.mark_text())
                .filter(|mark| !mark.is_empty())
                .unwrap_or("UI");
            let brand_title_text = branding
                .map(|brand| brand.title())
                .unwrap_or_else(|| application.window_config().title());
            let brand_tagline_text = branding
                .map(|brand| brand.tagline_text())
                .filter(|tagline| !tagline.is_empty())
                .unwrap_or("Application workspace");
            let brand_mark = create_static_label(
                hwnd, instance, BRAND_MARK_CONTROL_ID, brand_mark_text, SS_CENTER,
            );
            let brand_title = create_static_label(
                hwnd, instance, BRAND_TITLE_CONTROL_ID, brand_title_text, 0,
            );
            let brand_tagline = create_static_label(
                hwnd, instance, BRAND_TAGLINE_CONTROL_ID, brand_tagline_text, 0,
            );
            let page_title = create_static_label(hwnd, instance, PAGE_TITLE_CONTROL_ID, "", 0);
            let page_description =
                create_static_label(hwnd, instance, PAGE_DESCRIPTION_CONTROL_ID, "", 0);

            if navigation.is_null()
                || content.is_null()
                || brand_mark.is_null()
                || brand_title.is_null()
                || brand_tagline.is_null()
                || page_title.is_null()
                || page_description.is_null()
            {
                return -1;
            }

            let action_buttons = [
                create_action_button(
                    hwnd,
                    instance,
                    LOCATE_ENGINE_BUTTON_ID,
                    "Locate Wallpaper Engine",
                ),
                create_action_button(
                    hwnd,
                    instance,
                    BROWSE_WALLPAPER_BUTTON_ID,
                    "Browse Wallpaper",
                ),
                create_action_button(hwnd, instance, WORKSHOP_BUTTON_ID, "Open Steam Workshop"),
                create_action_button(hwnd, instance, OPENVERSE_BUTTON_ID, "Open Openverse"),
                create_action_button(hwnd, instance, COMMONS_BUTTON_ID, "Open Wikimedia Commons"),
                create_action_button(hwnd, instance, NASA_BUTTON_ID, "Open NASA Images"),
                create_action_button(
                    hwnd,
                    instance,
                    REFRESH_DISPLAYS_BUTTON_ID,
                    "Refresh Displays",
                ),
            ];
            if action_buttons.iter().any(|button| button.is_null()) {
                return -1;
            }

            unsafe {
                (*state).navigation = navigation;
                (*state).content = content;
                (*state).brand_mark = brand_mark;
                (*state).brand_title = brand_title;
                (*state).brand_tagline = brand_tagline;
                (*state).page_title = page_title;
                (*state).page_description = page_description;
                (*state).action_buttons = action_buttons;
                if !(*state).font.is_null() {
                    SendMessageW(navigation, WM_SETFONT, (*state).font as usize, 1);
                    SendMessageW(content, WM_SETFONT, (*state).font as usize, 1);
                    SendMessageW(brand_tagline, WM_SETFONT, (*state).font as usize, 1);
                    SendMessageW(page_description, WM_SETFONT, (*state).font as usize, 1);
                    if !(*state).brand_font.is_null() {
                        SendMessageW(brand_mark, WM_SETFONT, (*state).brand_font as usize, 1);
                        SendMessageW(brand_title, WM_SETFONT, (*state).brand_font as usize, 1);
                    }
                    if !(*state).heading_font.is_null() {
                        SendMessageW(page_title, WM_SETFONT, (*state).heading_font as usize, 1);
                    }
                    for button in (*state).action_buttons.iter() {
                        SendMessageW(*button, WM_SETFONT, (*state).font as usize, 1);
                    }
                }
                for button in (*state).action_buttons.iter() {
                    ShowWindow(*button, SW_HIDE);
                }
            }

            for page in application.pages() {
                let title = wide(page.navigation_label());
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
                set_page_content(&*state, &application.pages()[initial_index]);
                update_action_buttons(&*state, &application.pages()[initial_index]);
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
                        let content_x = navigation_width + 32;
                        let content_width = width.saturating_sub(navigation_width + 48);
                        MoveWindow((*state).brand_mark, 16, 12, 40, 40, 1);
                        MoveWindow((*state).brand_title, 64, 10, 240, 26, 1);
                        MoveWindow((*state).brand_tagline, 64, 36, 240, 18, 1);
                        MoveWindow(
                            (*state).navigation,
                            16,
                            64,
                            navigation_width,
                            height.saturating_sub(80),
                            1,
                        );
                        MoveWindow((*state).page_title, content_x, 64, content_width, 34, 1);
                        MoveWindow((*state).page_description, content_x, 99, content_width, 34, 1);
                        MoveWindow(
                            (*state).content,
                            content_x,
                            140,
                            content_width,
                            height.saturating_sub(156),
                            1,
                        );
                        let action_width = ((content_width.saturating_sub(16)) / 3).max(120);
                        for (index, button) in (*state).action_buttons.iter().enumerate() {
                            MoveWindow(
                                *button,
                                content_x + (index as i32 * (action_width + 8)),
                                16,
                                action_width,
                                32,
                                1,
                            );
                        }
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
                let selected = unsafe { SendMessageW((*state).navigation, LB_GETCURSEL, 0, 0) };
                if selected >= 0 {
                    if let Some(page) = application.pages().get(selected as usize) {
                        unsafe {
                            set_page_content(&*state, page);
                            update_action_buttons(&*state, page);
                        }
                    }
                }
            } else if !state.is_null() && notification == 0 {
                handle_store_command(state, control_id);
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

unsafe fn set_page_content(state: &WindowState, page: &UiPage) {
    let title = wide(page.title());
    let description = wide(page.description());
    unsafe {
        SetWindowTextW(state.page_title, title.as_ptr());
        SetWindowTextW(state.page_description, description.as_ptr());
    }
    let mut body = String::new();
    let engine = state.wallpaper_engine.as_ref();
    let display_status = state.display_status.as_str();
    match page.id() {
        "home" => body.push_str(
            "\r\n\r\nWINLOOK DESKTOP ENGINE\r\nA unified workspace for wallpapers, displays and desktop modules.\r\n\r\nWLDE modules: Store provider hub, monitor snapshot API and resource-pressure policy are available. Native rendering, decoding and widget execution are still in progress.",
        ),
        "wallpaper" => body.push_str(
            "\r\n\r\nWALLPAPER WORKSPACE\r\nOpen the Store, manage local wallpapers, review display topology or inspect Resource & Performance. Widget hosting, Game Mode detection and Groups are shown in their own sections.",
        ),
        "wallpaper-store" => body.push_str(
            "\r\n\r\nChoose Wallpaper Engine or browse the free catalogues. Providers keep their own terms, downloads and attribution requirements.",
        ),
        "wallpaper-store-wallpaper-engine" => {
            body.push_str("\r\n\r\n");
            body.push_str(&match engine {
                Some(install) => format!("Detected: {}", install.executable().display()),
                None => "Wallpaper Engine was not detected. Use Locate Wallpaper Engine to browse to wallpaper64.exe or wallpaper32.exe.".to_string(),
            });
            body.push_str(
                "\r\n\r\nUse Browse Wallpaper to select a project.json or supported media file. Wallpaper Engine should already be running.",
            );
            body.push_str(
                "\r\nSteam handles Workshop subscriptions, purchases, downloads and updates. WinLook does not unpack or redistribute Workshop packages.",
            );
        }
        "wallpaper-store-free" => body.push_str(
            "\r\n\r\nOpenverse: confirm the item's exact license, creator, source and attribution.\r\nWikimedia Commons: licenses vary by file; follow the file page.\r\nNASA: check third-party notices and credits; NASA endorsement is not implied.",
        ),
        "wallpaper-library" => body.push_str(
            "\r\n\r\nYOUR WALLPAPER COLLECTION\r\nA home for wallpapers you choose to keep offline. Local import, previews and collection persistence are not connected yet.\r\n\r\nCOLLECTIONS\r\nGroup wallpapers into named collections and retain source/license details with each item.\r\n\r\nRECENTLY ADDED\r\nThis area will show newly imported items when the local index is connected. No item counts or placeholder files are shown.\r\n\r\nSTORAGE & BACKUP\r\nThe future library store will preserve original media and keep generated thumbnails separate from source files.",
        ),
        "displays" => body.push_str(&format!(
            "\r\n\r\nDISPLAY SNAPSHOT\r\n{}\r\n\r\nA manual snapshot can be refreshed with the button above. Display-change hot-plug events, per-monitor DPI/orientation and wallpaper assignment are still being integrated.",
            display_status
        )),
        "wallpaper-performance" => body.push_str(
            "\r\n\r\nRESOURCE & PERFORMANCE\r\nThe Rust and C89 resource-pressure controllers are ready for CPU/GPU/RAM/VRAM samples. No OS telemetry provider is connected yet, so live percentages are not available. Optimize targets 30 FPS; Suspend stops animation requests.",
        ),
        "wallpaper-widgets" => body.push_str(
            "\r\n\r\nWIDGETS\r\nThe shared scene clock and capability contract are ready. No third-party widget code is executed until a restricted runtime, explicit permissions and crash isolation are implemented.",
        ),
        "icons" => body.push_str(
            "\r\n\r\nWLDE ADAPTIVE APPLICATION ICONS\r\nThe shared Rust and C89 cores now compose foreground/background app artwork into bounded, cached RGBA tiles with Circle, RoundedSquare or Squircle masks and safe-zone scaling. The optional monochrome layer uses its alpha channel as a glyph mask and accepts a caller-supplied theme tint. Source artwork, shortcuts and executables are never rewritten.\r\n\r\nNATIVE INTEGRATION\r\nShell icon extraction, DPI-aware cache policy and icon registration are still adapter work; this page does not imply that the live Explorer desktop has been rethemed yet.\r\n\r\nRIGHT-CLICK MODEL\r\nThe portable contract exposes Customize Icon for apps, add/remove-to-Group actions for item types, Group management actions, and Create Group on the desktop background. Native Open, Properties and normal Shell commands remain Windows-owned.",
        ),
        "wallpaper-groups" => body.push_str(
            "\r\n\r\nDESKTOP GROUPS\r\nGroups are virtual collections of app shortcuts, files and folders; they never create folders or move source items. The Rust model supports transactional create, rename, add/remove members and ungroup operations. The C89 model provides bounded caller-owned Group storage, validated replacement and lookup helpers.\r\n\r\nBGW INVENTORY\r\nBGW can take a read-only, one-level snapshot of the current User Desktop and Public Desktop Known Folders and assign opaque item IDs. Manager status IPC and durable local Group storage are not connected yet; virtual Shell namespace items and exact Explorer view membership still need the Shell-view adapter.\r\n\r\nPRESENTATION SAFETY\r\nLive Group tiles remain disabled until the OS adapter can restore original icon visibility and placement after crash, disable, session exit and uninstall.",
        ),
        "wallpaper-rainmeter" => body.push_str(
            "\r\n\r\nRAINMETER\r\nRMSKIN parsing is not runtime compatibility. Measures, meters, variables, actions, plugins, extraction safety and licensing still need a reviewed compatibility subset.",
        ),
        "game-mode" => body.push_str(
            "\r\n\r\nGAME MODE\r\nThe bounded SHA-256 per-application profile resolver is ready. Exact user profiles override optional OS Game Mode/fullscreen fallbacks. Win32 foreground/fullscreen signal collection and profile persistence are still pending.",
        ),
        _ => {},

    }
    let text = wide(&body);
    unsafe {
        SetWindowTextW(state.content, text.as_ptr());
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
    let cursor = unsafe { LoadCursorW(std::ptr::null_mut(), 32512usize as *const u16) };
    let class = WindowClass {
        style: 0x0002 | 0x0001,
        window_proc: Some(window_proc),
        class_extra: 0,
        window_extra: 0,
        instance,
        icon: std::ptr::null_mut(),
        cursor,
        background: std::ptr::null_mut(),
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

    let palette: &UiPalette = application.theme_ref().palette();
    let window_color =
        component_background(&application, UiComponent::Window).unwrap_or(palette.window);
    let navigation_color =
        component_background(&application, UiComponent::Navigation).unwrap_or(palette.surface_alt);
    let content_color =
        component_background(&application, UiComponent::Surface).unwrap_or(palette.surface);
    let text_color =
        component_foreground(&application, UiComponent::Text).unwrap_or(palette.text_primary);
    let state = WindowState {
        application: application.clone(),
        navigation: std::ptr::null_mut(),
        content: std::ptr::null_mut(),
        brand_mark: std::ptr::null_mut(),
        brand_title: std::ptr::null_mut(),
        brand_tagline: std::ptr::null_mut(),
        page_title: std::ptr::null_mut(),
        page_description: std::ptr::null_mut(),
        window_brush: unsafe { CreateSolidBrush(colorref(window_color)) },
        navigation_brush: unsafe { CreateSolidBrush(colorref(navigation_color)) },
        content_brush: unsafe { CreateSolidBrush(colorref(content_color)) },
        brand_brush: unsafe { CreateSolidBrush(colorref(palette.accent)) },
        font: create_font(&application),
        brand_font: create_font_at(
            &application,
            application.theme_ref().typography_tokens().size_lg,
            application.theme_ref().typography_tokens().weight_semibold,
        ),
        heading_font: create_font_at(
            &application,
            application.theme_ref().typography_tokens().size_xl,
            application.theme_ref().typography_tokens().weight_semibold,
        ),
        navigation_color,
        content_color,
        brand_color: palette.accent,
        brand_text_color: palette.accent_text,
        text_color,
        secondary_text_color: palette.text_secondary,
        action_buttons: [std::ptr::null_mut(); BUTTON_COUNT],
        wallpaper_engine: WallpaperEngineInstall::discover(),
        display_status: monitor_snapshot_status(),
    };
    if state.window_brush.is_null()
        || state.navigation_brush.is_null()
        || state.content_brush.is_null()
        || state.brand_brush.is_null()
        || state.font.is_null()
        || state.brand_font.is_null()
        || state.heading_font.is_null()
    {
        delete_state_resources(&state);
        return Err(UiError::Backend(
            "CreateSolidBrush failed while preparing the UI theme".to_string(),
        ));
    }
    let state_pointer = Box::into_raw(Box::new(state));
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
        unsafe {
            delete_state_resources(&*state_pointer);
            drop(Box::from_raw(state_pointer));
        }
        let error = UiError::Backend("CreateWindowExW failed for the native Win32 UI".to_string());
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
        let result = unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) };
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
    unsafe {
        delete_state_resources(&*state_pointer);
        drop(Box::from_raw(state_pointer));
    }

    if loop_error {
        let error =
            UiError::Backend("GetMessageW failed in the native Win32 message loop".to_string());
        show_start_error(&error, application.window_config().title());
        Err(error)
    } else {
        Ok(())
    }
}
