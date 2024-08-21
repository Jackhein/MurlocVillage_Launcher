#[cxx_qt::bridge]
mod my_object {

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
            use std::env;
            use std::path::Path;
            use sha2::{Sha512, Digest};
            use std::{io, fs};
            use serde_json::Value;
            let current_path = env::current_dir().unwrap().display().to_string();

            let data = match fs::read_to_string("resources/hashmap.json") {
                Ok(f) => f,
                Err(_e) => return QString::from("Resource file is missing.")
            };
            let json: Value = serde_json::from_str(&data).expect("Invalid JSON");
            if let Some(file_hashes) = json.get("core_files").and_then(|v| v.as_object()) {
                for (file, values) in file_hashes {
                    if let Some(array) = values.as_array() {
                        if array.len() == 2 {
                            let correct_hash = array[0].as_str().unwrap_or("");
                            let url = array[1].as_str().unwrap_or("");
                            let current_file = format!("{}/{}", current_path, file);

                            if !Path::new(&current_file).exists() {
                                return QString::from(&format!("File {} doesn't exist, download from {}.", file, url));
                            }
                            let mut hasher = Sha512::new();
                            let mut file_io = match fs::File::open(current_file) {
                                Ok(f) => f,
                                Err(_e) => return QString::from(&format!("File {} couldn't be opened.", file))
                            };

                            let _ = io::copy(&mut file_io, &mut hasher);
                            return if hex::encode(hasher.finalize()) == correct_hash {
                                QString::from(&format!("File {} is correct.", file))
                            } else {
                                QString::from(&format!("File {} isn't correct, download from {}.", file, url))
                            }
                        } else {
                            return QString::from(&format!("File {} data in JSON are corrupted (wrong array size)", file));
                        }
                    } else {
                        return QString::from(&format!("File {} data in JSON are corrupted (not an array)", file));
                    }
                }
            }
            QString::from("JSON isn't valid")
        }
    }
}
