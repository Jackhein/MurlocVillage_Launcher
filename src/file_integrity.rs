#[cxx_qt::bridge]
mod qobject {
    // Already linked through #[cxx_qt::bridge], needed to dismiss error
    use crate::file_integrity::qobject::FileIntegrityCxxQtThread;
    // Already linked in unsafe extern "C++", needed to dismiss error
    use cxx_qt_lib::QString;
    use serde_json::Value;
    use sha2::{Digest, Sha512};
    use std::{env, fs, io};
    use std::{env::consts::OS, process::{Command, Stdio}};
    use std::{path::Path, time::Duration};
    use std::{sync::Arc, thread};
    use tokio::{io::AsyncWriteExt, time::sleep};

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[cxx_qt::qobject(qml_uri = "WoW_Private_Server_Launcher", qml_version = "1.0")]
    pub struct FileIntegrity {
        #[qproperty]
        title: QString,
        #[qproperty]
        play: QString,
        #[qproperty]
        verify: QString,
        #[qproperty]
        language: QString,
        #[qproperty]
        result: QString,
    }

    impl Default for FileIntegrity {
        fn default() -> Self {
            Self {
                title: qobject::FileIntegrity::load_gui_title(),
                play: qobject::FileIntegrity::load_gui_button_play(),
                verify: qobject::FileIntegrity::load_gui_button_verify(),
                language: qobject::FileIntegrity::load_gui_button_language(),
                result: QString::from(""),
            }
        }
    }

    impl qobject::FileIntegrity {
        /// Repair / Download the game Core
        #[qinvokable]
        pub fn check_file(self: Pin<&mut Self>) {
            let qt_thread = self.qt_thread();

            thread::spawn(move || {
                qobject::FileIntegrity::display_message(&qt_thread, "Begin check_file.");
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let current_path = env::current_dir().unwrap().display().to_string();
                let data = match fs::read_to_string("resources/hashmap.json") {
                    Ok(f) => f,
                    Err(_e) => {
                        qobject::FileIntegrity::display_message(&qt_thread, "Resource file is missing.");
                        return
                    },
                };
                let json: Value = serde_json::from_str(&data).expect("Invalid JSON");
                if let Some(file_hashes) = json.get("core_files").and_then(|v| v.as_object()) {
                    return for (i, (file, values)) in file_hashes.iter().enumerate() {
                        qobject::FileIntegrity::display_message(&qt_thread, &format!("Current file: {} on {}", i, file_hashes.len()));
                        if let Some(array) = values.as_array() {
                            if array.len() == 2 {
                                let correct_hash = array[0].as_str().unwrap_or("");
                                let url = array[1].as_str().unwrap_or("");
                                let current_file = format!("{}/{}", current_path, file);

                                if !Path::new(&current_file).exists() {
                                    if runtime.block_on(qobject::FileIntegrity::download_file_process(url, &current_file, &qt_thread)) {
                                        qobject::FileIntegrity::display_message(&qt_thread, &format!("Unable to download {} from {}.", file, url));
                                        return
                                    }
                                    qobject::FileIntegrity::display_message(&qt_thread, &format!("File {} didn't exist, downloaded from {}.", file, url));
                                }
                                let mut hasher = Sha512::new();
                                let mut file_io = match fs::File::open(&current_file) { // or use of .clone()
                                    Ok(f) => f,
                                    Err(_e) => {
                                        qobject::FileIntegrity::display_message(&qt_thread, &format!("File {} couldn't be opened.", file));
                                        return
                                    }
                                };

                                let _ = io::copy(&mut file_io, &mut hasher);
                                if hex::encode(hasher.clone().finalize()) == correct_hash {
                                    qobject::FileIntegrity::display_message(&qt_thread, &format!("File {} is correct.", file));
                                } else {
                                    if fs::remove_file(&current_file).is_err() {
                                        qobject::FileIntegrity::display_message(&qt_thread, &format!("Unable to delete {}.", file));
                                        return
                                    } else if runtime.block_on(qobject::FileIntegrity::download_file_process(url, &current_file, &qt_thread)) {
                                        qobject::FileIntegrity::display_message(&qt_thread, &format!("Unable to download {} from {}.", file, url));
                                        return
                                    }
                                    qobject::FileIntegrity::display_message(&qt_thread, &format!("File {} isn't correct ({}), download from {}.", file, hex::encode(hasher.clone().finalize()), url));
                                }
                            } else {
                                qobject::FileIntegrity::display_message(&qt_thread, &format!("File {} data in JSON are corrupted (wrong array size)", file));
                                return
                            }
                        } else {
                            qobject::FileIntegrity::display_message(&qt_thread, &format!("File {} data in JSON are corrupted (not an array)", file));
                        }
                    };
                }
                qobject::FileIntegrity::display_message(&qt_thread, "JSON isn't valid");
            });
        }

        /// Start the game
        #[qinvokable]
        pub fn start_game(&self) -> QString {
            if OS == "linux" || OS == "macos" {
                match Command::new("wine")
                    .arg("Wow.exe")
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn() {
                    Ok(_) => QString::from("Game started"),
                    Err(e) => QString::from(&format!("Unable to launch game: {}", e))
                }
            } else if OS == "windows" {
                match Command::new("Wow.exe")
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn() {
                    Ok(_) => crate::file_integrity::qobject::QString::from("Game started"),
                    Err(e) => crate::file_integrity::qobject::QString::from(&format!("Unable to launch game: {}", e))
                }
            } else {
                QString::from(&format!("Unsupported OS: {}.", OS))
            }
        }

        /// GUI information update
        fn display_message(qt_thread: &UniquePtr<FileIntegrityCxxQtThread>, msg: &str) {
            println!("{}", msg);
            let queued_msg = Arc::new(msg.to_string());
            qt_thread.queue(move | qobject| {
                qobject.set_result(QString::from(queued_msg.as_str()));
            }).unwrap();
        }

        fn download_message(qt_thread: &UniquePtr<FileIntegrityCxxQtThread>, msg: &str) {
            let queued_msg = Arc::new(msg.to_string());
            qt_thread.queue(move | qobject| {
                qobject.set_result(QString::from(queued_msg.as_str()));
            }).unwrap();
        }

        /// Download procedure
        async fn download_file(url: &str, current_file: &str, qt_thread: &UniquePtr<FileIntegrityCxxQtThread>) -> Result<(), String> {
            let mut response = reqwest::get(url).await.map_err(|_| "DownloadFailure")?;

            let total_size = response.content_length().unwrap_or(0);
            let mut downloaded = 0;
            while let Some(chunk) = response.chunk().await.map_err(|_| "DownloadFailure")? {
                downloaded += chunk.len() as u64;
                qobject::FileIntegrity::download_message(qt_thread, &format!("File downloading: {}%", 100 * downloaded / total_size));
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

        async fn download_file_process(url: &str, current_file: &str, qt_thread: &UniquePtr<FileIntegrityCxxQtThread>) -> bool {
            let mut retry = 0;
            let max_retries = 3;

            while retry < max_retries {
                retry += 1;
                match qobject::FileIntegrity::download_file(url, current_file, qt_thread).await {
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
        fn load_gui_button_play() -> QString {
            QString::from("Play")
        }

        fn load_gui_button_verify() -> QString {
            QString::from("Verify game")
        }

        fn load_gui_button_language() -> QString {
            QString::from("Language settings")
        }

        fn load_gui_title() -> QString {
            QString::from("WoW Private Server Launcher")
        }
        // /// Download the add-on

    }
}
