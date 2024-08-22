#[cxx_qt::bridge]
mod my_object {
    use sha2::{Sha512, Digest};
    use std::{env, path::Path, io, fs, time::Duration};
    use serde_json::Value;
    use tokio::{time::sleep, io::AsyncWriteExt};

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[cxx_qt::qobject(qml_uri = "MurlocVillage_Launcher", qml_version = "1.0")]
    pub struct FileIntegrity {
        #[qproperty]
        file: QString,
        #[qproperty]
        result: QString,
    }

    impl Default for FileIntegrity {
        fn default() -> Self {
            Self {
                file: QString::from(""),
                result: QString::from(""),
            }
        }
    }

    impl qobject::FileIntegrity {
        #[qinvokable]
        pub fn check_file(&self) -> QString {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let current_path = env::current_dir().unwrap().display().to_string();
            let data = match fs::read_to_string("resources/hashmap.json") {
                Ok(f) => f,
                Err(_e) => return QString::from("Resource file is missing.")
            };
            let json: Value = serde_json::from_str(&data).expect("Invalid JSON");
            if let Some(file_hashes) = json.get("core_files").and_then(|v| v.as_object()) {
                for (file, values) in file_hashes {
                    return if let Some(array) = values.as_array() {
                        if array.len() == 2 {
                            let correct_hash = array[0].as_str().unwrap_or("");
                            let url = array[1].as_str().unwrap_or("");
                            let current_file = format!("{}/{}", current_path, file);

                            if !Path::new(&current_file).exists() {
                                if runtime.block_on(self.download_file_process(url, current_file)) {
                                    return QString::from(&format!("Unable to download {} from {}.", file, url));
                                }
                                return QString::from(&format!("File {} didn't exist, downloaded from {}.", file, url));
                            }
                            let mut hasher = Sha512::new();
                            let mut file_io = match fs::File::open(&current_file) { // or use of .clone()
                                Ok(f) => f,
                                Err(_e) => return QString::from(&format!("File {} couldn't be opened.", file))
                            };

                            let _ = io::copy(&mut file_io, &mut hasher);
                            if hex::encode(hasher.finalize()) == correct_hash {
                                QString::from(&format!("File {} is correct.", file))
                            } else {
                                if fs::remove_file(&current_file).is_err() {
                                    return QString::from(&format!("Unable to delete {}.", file));
                                } else if runtime.block_on(self.download_file_process(url, current_file)) {
                                    return QString::from(&format!("Unable to download {} from {}.", file, url));
                                }
                                QString::from(&format!("File {} isn't correct, download from {}.", file, url))
                            }
                        } else {
                            QString::from(&format!("File {} data in JSON are corrupted (wrong array size)", file))
                        }
                    } else {
                        QString::from(&format!("File {} data in JSON are corrupted (not an array)", file))
                    }
                }
            }
            QString::from("JSON isn't valid")
        }

        async fn download_file(&self, url: &str, current_file: String) -> Result<(), String> {
            let response = reqwest::get(url).await.map_err(|_| "DownloadFailure")?;
            if !response.status().is_success() {
                //Err(reqwest::Error::new(reqwest::StatusCode::from(response.status()), "Failed to download file"));
            }
            let content = response.bytes().await.map_err(|_| "DownloadFailure")?;

            let path = Path::new(&current_file);
            let mut file = tokio::fs::File::create(&path).await.map_err(|_| "DownloadFailure")?;

            file.write_all(&content).await.map_err(|_| "DownloadFailure")?;
            Ok(())
        }

        async  fn download_file_process(&self, url: &str, current_file: String) -> bool {
            let mut retry = 0;
            let max_retries = 3;

            while retry < max_retries {
                retry += 1;
                match self.download_file(url, current_file.clone()).await {
                    Ok(_) => {
                        println!("File downloaded successfully!");
                        return false;
                    },
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
    }
}
