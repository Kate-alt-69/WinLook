#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = wl_bgw_runtime::run() {
        wl_bgw_runtime::report_fatal(&format!("{error}"));
        std::process::exit(1);
    }
}
