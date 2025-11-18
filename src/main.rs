#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod windows;
mod wrap_app;

use eframe;
use egui::ViewportBuilder;

use wrap_app::WrapApp;
use crate::windows::settings::{compute_window_size, MAIN_HEIGHT};

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_decorations(false)
            .with_resizable(true)
            // .with_inner_size([., Y+50.]),
            .with_inner_size(compute_window_size(MAIN_HEIGHT)),
        ..Default::default()
    };

    // eframe::run_native(
    //     "test extra",
    //     native_options,
    //     Box::new(|cc| {
    //         egui_extras::install_image_loaders(&cc.egui_ctx);

    //         let style=Style{
    //             visuals: Visuals::dark(),
    //             ..Default::default()
    //         };
    //         cc.egui_ctx.set_style(style);
    //         Ok(Box::<main_app::MainApp>::default())
    //     }),
    // )

    eframe::run_native(
        "test",
        native_options,
        Box::new(|cc| Ok(Box::new(WrapApp::new(cc)))),
    )
}
