#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = wl_ui::windows10::run() {
        eprintln!("WinLook Windows 10 Manager failed to start: {error}");
        std::process::exit(1);
    }
}
