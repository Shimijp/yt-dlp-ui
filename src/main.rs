#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ui;
mod downloader;


use crate::ui::App;


fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Download manager",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
        .expect("failed to run program");



}
