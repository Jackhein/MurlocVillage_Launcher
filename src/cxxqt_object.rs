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
        pub fn file_exist(&self, file: &QString) -> QString {
            use std::path::Path;
            use std::env;

            let current_file = &format!("{}/{}", env::current_dir().unwrap().display(), file);

            let result = format!("{current_file:?} {}exist", if Path::new(current_file).exists()  { "" } else { "doesn't" });
            QString::from(&result)
        }
    }
}
