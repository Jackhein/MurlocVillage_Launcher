#[cxx_qt::bridge(qml_uri = "WoW_Private_Server_Launcher", qml_version = "1.0")]
mod file_integrity {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qimage.h");
        type QImage = cxx_qt_lib::QImage;
        // include!("cxx-qt-lib/qpixmap.h");
        // type QPixmap = cxx_qt_lib::QPixmap;
        // include!("cxx-qt-lib/qsound.h");
        // type QSound = cxx_qt_lib::QSound;
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, title)]
        #[qproperty(QString, play)]
        #[qproperty(QString, verify)]
        #[qproperty(QString, language)]
        // #[qproperty(QImage, font)]
        // #[qproperty(QImage, loading_icon)]
        // #[qproperty(QImage, loading_bar)]
        #[qproperty(QColor, button_color)]
        #[qproperty(QString, result)]
        type FileIntegrity = super::FileIntegrityRust;
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        pub fn check_file(self: Pin<&mut FileIntegrity>);
        #[qinvokable]
        pub fn start_game(self: Pin<&mut FileIntegrity>) -> QString;
    }

    impl cxx_qt::Threading for FileIntegrity {}
}

use cxx_qt::{CxxQtThread, Threading};
use serde_json::Value;
use sha2::{Digest, Sha512};
use std::{env, fs, io, pin::Pin};
use std::{
    env::consts::OS,
    io::Read,
    process::{Command, Stdio},
};
use std::{path::Path, time::Duration};
use std::{sync::Arc, thread};
use cxx_qt_lib::{QColor, QImage, QString};
use tokio::{io::AsyncWriteExt, time::sleep};

/// Rust structure bridged to C++ code.
pub struct FileIntegrityRust {
    title: QString,
    play: QString,
    verify: QString,
    language: QString,
    // font: QImage,
    // loading_icon: QImage,
    // loading_bar: QImage,
    button_color: QColor,
    result: QString,
}

/// Default data for structure.
/// Can use #[derive(Default)] instead ahead of struct declaration.
/// Check for future use cxx_qt::{Constructor, Initialize}.
impl Default for FileIntegrityRust {
    fn default() -> Self {
        Self {
            title: file_integrity::FileIntegrity::load_gui_text_title(),
            play: file_integrity::FileIntegrity::load_gui_button_text_play(),
            verify: file_integrity::FileIntegrity::load_gui_button_text_verify(),
            language: file_integrity::FileIntegrity::load_gui_button_text_language(),
            // font: file_integrity::FileIntegrity::load_gui_font(),
            // loading_icon: file_integrity::FileIntegrity::load_gui_loading_icon(),
            // loading_bar: file_integrity::FileIntegrity::load_gui_loading_bar(),
            button_color: file_integrity::FileIntegrity::load_gui_button_color(),
            result: QString::from(""),
        }
    }
}


impl file_integrity::FileIntegrity {
    /// Repair / Download the game Core
    pub fn check_file(self: Pin<&mut Self>) {
        let qt_thread = self.qt_thread();

        thread::spawn(move || {
            Self::display_message(&qt_thread, "Begin check_file.");
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let current_path = env::current_dir().unwrap().display().to_string();
            let data = match fs::read_to_string("resources/hashmap.json") {
                Ok(f) => f,
                Err(_e) => {
                    Self::display_message(
                        &qt_thread,
                        "Resource file is missing.",
                    );
                    return;
                }
            };
            let json: Value = serde_json::from_str(&data).expect("Invalid JSON");
            if let Some(file_hashes) = json.get("core_files").and_then(|v| v.as_object()) {
                return for (i, (file, values)) in file_hashes.iter().enumerate() {
                    Self::display_message(
                        &qt_thread,
                        &format!("Current file: {} on {}", i, file_hashes.len()),
                    );
                    if let Some(array) = values.as_array() {
                        if array.len() == 2 {
                            let correct_hash = array[0].as_str().unwrap_or("");
                            let url = array[1].as_str().unwrap_or("");
                            let current_file = format!("{}/{}", current_path, file);

                            if !Path::new(&current_file).exists() {
                                if runtime.block_on(
                                    Self::download_file_process(
                                        url,
                                        &current_file,
                                        &qt_thread,
                                    ),
                                ) {
                                    Self::display_message(
                                        &qt_thread,
                                        &format!("Unable to download {} from {}.", file, url),
                                    );
                                    return;
                                }
                                Self::display_message(
                                    &qt_thread,
                                    &format!(
                                        "File {} didn't exist, downloaded from {}.",
                                        file, url
                                    ),
                                );
                            }
                            let mut hasher = Sha512::new();
                            let mut file_io = match fs::File::open(&current_file) {
                                Ok(f) => f,
                                Err(_e) => {
                                    Self::display_message(
                                        &qt_thread,
                                        &format!("File {} couldn't be opened.", file),
                                    );
                                    return;
                                }
                            };

                            let _ = io::copy(&mut file_io, &mut hasher);
                            if hex::encode(hasher.clone().finalize()) == correct_hash {
                                Self::display_message(
                                    &qt_thread,
                                    &format!("File {} is correct.", file),
                                );
                            } else {
                                if fs::remove_file(&current_file).is_err() {
                                    Self::display_message(
                                        &qt_thread,
                                        &format!("Unable to delete {}.", file),
                                    );
                                    return;
                                } else if runtime.block_on(
                                    Self::download_file_process(
                                        url,
                                        &current_file,
                                        &qt_thread,
                                    ),
                                ) {
                                    Self::display_message(
                                        &qt_thread,
                                        &format!("Unable to download {} from {}.", file, url),
                                    );
                                    return;
                                }
                                Self::display_message(
                                    &qt_thread,
                                    &format!(
                                        "File {} isn't correct ({}), download from {}.",
                                        file,
                                        hex::encode(hasher.clone().finalize()),
                                        url
                                    ),
                                );
                            }
                        } else {
                            Self::display_message(
                                &qt_thread,
                                &format!(
                                    "File {} data in JSON are corrupted (wrong array size)",
                                    file
                                ),
                            );
                            return;
                        }
                    } else {
                        Self::display_message(
                            &qt_thread,
                            &format!("File {} data in JSON are corrupted (not an array)", file),
                        );
                    }
                };
            }
            Self::display_message(&qt_thread, "JSON isn't valid");
        });
    }

    /// Start the game
    pub fn start_game(self: Pin<&mut Self>) -> QString {
        let qt_thread = self.qt_thread();

        thread::spawn(move || {
            return if OS == "linux" || OS == "macos" {
                match Self::run_game("wine", "Wow.exe", &qt_thread) {
                    Ok(_) => QString::from("Game started"),
                    Err(e) => QString::from(&format!("Unable to launch game: {}", e)),
                }
            } else if OS == "windows" {
                match Self::run_game("Wow.exe", "", &qt_thread) {
                    Ok(_) => QString::from("Game started"),
                    Err(e) => QString::from(&format!("Unable to launch game: {}", e)),
                }
            } else {
                QString::from(&format!("Unsupported OS: {}.", OS))
            };
        });
        QString::from("Game launching")
    }

    fn run_game(
        cmd: &str,
        args: &str,
        qt_thread: &CxxQtThread<Self>,
    ) -> Result<(), String> {
        let mut thread_game = Command::new(cmd)
            .arg(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| "GameFailure")?;
        if let Some(mut stderr) = thread_game.stderr.take() {
            let mut stderr_output = String::new();
            stderr.read_to_string(&mut stderr_output).map_err(|_| "GameFailure")?;

            if !stderr_output.is_empty() {
                Self::display_message(
                    &qt_thread,
                    &format!("Error from wine: {}", stderr_output),
                );
            }
        }
        Ok(())
    }

    /// GUI information update
    fn display_message(qt_thread: &CxxQtThread<Self>, msg: &str) {
        println!("{}", msg);
        let queued_msg = Arc::new(msg.to_string());
        qt_thread
            .queue(move |qobject| {
                qobject.set_result(QString::from(queued_msg.as_str()));
            })
            .unwrap();
    }

    fn download_message(qt_thread: &CxxQtThread<Self>, msg: &str) {
        let queued_msg = Arc::new(msg.to_string());
        qt_thread
            .queue(move |qobject| {
                qobject.set_result(QString::from(queued_msg.as_str()));
            })
            .unwrap();
    }

    /// Download procedure
    async fn download_file(
        url: &str,
        current_file: &str,
        qt_thread: &CxxQtThread<Self>,
    ) -> Result<(), String> {
        let mut response = reqwest::get(url).await.map_err(|_| "DownloadFailure")?;

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded = 0;
        while let Some(chunk) = response.chunk().await.map_err(|_| "DownloadFailure")? {
            downloaded += chunk.len() as u64;
            Self::download_message(
                &qt_thread,
                &format!("File downloading: {}%", 100 * downloaded / total_size),
            );
        }
        if !response.status().is_success() {
            //Err(reqwest::Error::new(reqwest::StatusCode::from(response.status()), "Failed to download file"));
        }
        let content = response.bytes().await.map_err(|_| "DownloadFailure")?;

        let path = Path::new(&current_file);
        let mut file = tokio::fs::File::create(&path).await.map_err(|_| "DownloadFailure")?;

        file.write_all(&content).await.map_err(|_| "DownloadFailure")?;
        Ok(())
    }

    async fn download_file_process(
        url: &str,
        current_file: &str,
        qt_thread: &CxxQtThread<Self>,
    ) -> bool {
        let mut retry = 0;
        let max_retries = 3;

        while retry < max_retries {
            retry += 1;
            match Self::download_file(url, current_file, &qt_thread).await {
                Ok(_) => {
                    println!("File downloaded successfully!");
                    return false;
                }
                Err(e) => {
                    eprintln!("Attempt {} failed: {}", retry, e);
                    if retry < max_retries {
                        println!("Retrying in 2 seconds...");
                        sleep(Duration::from_secs(2)).await;
                    } else {
                        eprintln!("All attempts failed. Giving up.");
                        return true;
                    }
                }
            }
        }
        true
    }

    /// Load the GUI text
    fn load_gui_button_text_play() -> QString {
        QString::from("Play")
    }

    fn load_gui_button_text_verify() -> QString {
        QString::from("Verify game")
    }

    fn load_gui_button_text_language() -> QString {
        QString::from("Language settings")
    }

    fn load_gui_text_title() -> QString {
        QString::from("WoW Private Server Launcher")
    }

    pub fn load_gui_button_color() -> QColor {
        QColor::from_rgba(155, 0, 0, 255)
    }

    // fn load_gui_font() -> QImage {
    //     QImage::from("test")
    // }
    //
    // fn load_gui_loading_icon() -> QImage {
    //     QImage::from("test")
    // }
    //
    //
    // fn load_gui_loading_bar() -> QImage {
    //     QImage::from("test")
    // }
    // /// Download the add-on
}
