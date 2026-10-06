use std::sync::mpsc::{Receiver, Sender, channel};

use eframe::egui;

use crate::downloader::{DownloadMsg, start_download};
use egui_file_dialog::FileDialog;

#[derive(PartialEq)]
pub struct Download {
    pub(crate) url: String,
    pub(crate) name: String,
    pub progress: f32,
    pub status_text: String,
    pub finished: bool,
}
impl Download {
    pub fn new(url: String, name: String) -> Self {
        Download {
            url,
            name,
            progress: 0.0,
            status_text: "Starting...".to_string(),
            finished: false,
        }
    }
}
pub struct App {
    current_url: String,
    format: String,
    downloads: Vec<Download>,
    download_path: String,
    file_dialog: FileDialog,
    tx: Sender<DownloadMsg>,
    rx: Receiver<DownloadMsg>,
}

impl App {
    pub fn update_downloads(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                DownloadMsg::Metadata { url, name } => {
                    if let Some(dl) = self.downloads.iter_mut().find(|d| d.url == url) {
                        dl.name = name;
                    }
                }
                DownloadMsg::Progress { url, percent, text } => {
                    if let Some(dl) = self.downloads.iter_mut().find(|d| d.url == url) {
                        dl.progress = percent;
                        dl.status_text = text;
                    }
                }
                DownloadMsg::Finished { url } => {
                    if let Some(dl) = self.downloads.iter_mut().find(|d| d.url == url) {
                        dl.finished = true;
                        dl.progress = 1.0;
                        dl.status_text = "Finished".to_string();
                    }
                }
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let (tx, rx) = channel();
        App {
            current_url: String::new(),
            format: String::new(),
            downloads: Vec::new(),
            download_path: std::env::current_dir()
                .expect("Failed to get current directory")
                .to_str()
                .unwrap()
                .to_string(),
            file_dialog: FileDialog::new(),
            tx,
            rx,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("download manager");
            ui.add_space(10.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label("Download Settings");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.current_url).hint_text("enter url"),
                    );

                    if ui.button("paste").clicked() {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            if let Ok(text) = clipboard.get_text() {
                                self.current_url = text;
                            }
                        }
                    }
                });
                ui.add(egui::TextEdit::singleline(&mut self.format).hint_text("specific format"));

                if ui.button("download").clicked() {
                    let current =
                        Download::new(self.current_url.to_string(), "Fetching data...".to_string());
                    start_download(
                        self.download_path.clone(),
                        current.url.clone(),
                        "".to_string(),
                        self.tx.clone(),
                    );
                    self.downloads.push(current);
                }
                ui.horizontal(|ui| {
                    if ui.button("clear url").clicked() {
                        self.current_url.clear();
                        self.format.clear();
                    }
                    if ui.button("clear list").clicked() {
                        self.downloads.retain(|x| !x.finished);
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("select download folder").clicked() {
                        self.file_dialog.pick_directory();
                    }
                    ui.label(&self.download_path);
                });

                self.file_dialog.update(ui);

                if let Some(path) = self.file_dialog.take_picked() {
                    self.download_path = path.to_str().unwrap().to_string();
                }
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for item in &self.downloads {
                        ui.horizontal(|ui| {
                            if item.progress < 1.0 {
                                ui.label(&item.name);
                                if item.progress > 0.0 {
                                    ui.add(
                                        egui::ProgressBar::new(item.progress)
                                            .text(&item.status_text),
                                    );
                                }
                            } else {
                                ui.colored_label(egui::Color32::GREEN, &item.name);

                                ui.colored_label(egui::Color32::GREEN, &item.status_text);
                            }
                        });
                    }
                });
            });

            self.update_downloads();
        });

        ui.ctx().request_repaint();
    }
}
