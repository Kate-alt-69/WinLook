#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = wl_ui::windows11::run() {
        eprintln!("WinLook Windows 11 Manager failed to start: {error}");
        std::process::exit(1);
    }
}
