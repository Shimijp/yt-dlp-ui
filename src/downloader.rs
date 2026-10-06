pub const YT_DLP_PATH : &str = r"PATH/TO_YOUR/yt-dlp.exe";




#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;


pub enum DownloadMsg {
    Metadata { url: String, name: String },
    Progress { url: String, percent: f32, text: String },
    Finished { url: String },
}

pub fn start_download(dir: String, url: String, args: String, tx: Sender<DownloadMsg>) {
    thread::spawn(move || {

        let mut meta_cmd = Command::new(YT_DLP_PATH);
        meta_cmd.arg("--print").arg("%(title)s").arg(&url);
        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            meta_cmd.creation_flags(CREATE_NO_WINDOW);
        }

        if let Ok(output) = meta_cmd.output() {
            if output.status.success() {
                let title = String::from_utf8_lossy(&output.stdout).trim().to_string();

                let _ = tx.send(DownloadMsg::Metadata { url: url.clone(), name: title });
            }
        }
        let mut cmd = Command::new(YT_DLP_PATH);

        cmd.current_dir(&dir);
        cmd.arg("--newline");

        if !args.is_empty() {
            cmd.arg("-f").arg(&args);
        }

        cmd.arg(&url).stdout(Stdio::piped());
        #[cfg(target_os = "windows")]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        if let Ok(mut child) = cmd.spawn() {
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);

                for line in reader.lines().flatten() {

                    if line.starts_with("[download]") && line.contains('%') {
                        let text = line.trim_start_matches("[download]").trim();


                        let mut percent_val = 0.0;
                        if let Some(idx) = text.find('%') {
                            let p_str = text[..idx].trim();
                            if let Ok(p) = p_str.parse::<f32>() {
                                percent_val = p / 100.0;
                            }
                        }

                        let _ = tx.send(DownloadMsg::Progress {
                            url: url.clone(),
                            percent: percent_val,
                            text: text.to_string(),
                        });
                    }
                }
            }
            let _ = child.wait();
        }

        let _ = tx.send(DownloadMsg::Finished { url });
    });
}