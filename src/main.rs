#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod windows;
mod wrap_app;

use eframe;
use egui::ViewportBuilder;
use wrap_app::WrapApp;
use crate::windows::settings::{compute_window_size, MAIN_HEIGHT};

/// start function. The window works on the principle of 5x3 rects.
/// main window is wrap_app, then in hear is main_app, doc_app and setting.
/// math dir for difraction and other calculation.

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_decorations(false)
            .with_resizable(true)
            // .with_inner_size([., Y+50.]),
            .with_inner_size(compute_window_size(MAIN_HEIGHT)),
        ..Default::default()
    };

    eframe::run_native(
        "test",
        native_options,
        Box::new(|cc| Ok(Box::new(WrapApp::new(cc)))),
    )
}
