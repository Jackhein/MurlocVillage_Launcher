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
        pub fn check_file(&self, file: &QString) -> QString {
            use std::path::Path;
            use std::env;
            use sha2::{Sha512, Digest};
            use std::{io, fs};

            let current_file = &format!("{}/{}", env::current_dir().unwrap().display(), file);

            if !Path::new(current_file).exists() {
                return QString::from(&format!("File {} doesn't exist.", current_file))
            }

            let mut hasher = Sha512::new();
            let mut file = match fs::File::open(current_file) {
                Ok(f) => f,
                Err(_e) => return QString::from(&format!("File {} couldn't be opened.", current_file)),
            };

            let _ = io::copy(&mut file, &mut hasher);
            QString::from(&(hex::encode(hasher.finalize())))
        }
    }
}
