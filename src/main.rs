#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
use app::*;
mod coins;
pub use coins::*;

fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Verteidigung Doktorarbeit Yannick Feld",
        native_options,
        Box::new(|cc| Box::new(AppState::new(cc))),
    )
}