mod file_integrity;

use std::{fs, path::Path};
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};
use serde_json::Value;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() {
    // Verify if resources are valid
    if check_resource().await {
        eprintln!("Failed to check resource, verify your /resource/hashmap.json file.")
    }

    // Create the application and engine
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    // Load the QML path into the engine
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from_local_file(&QString::from("qml/main.qml")));
    } else {
        eprintln!("Failed to load QQmlApplicationEngine.")
    }

    // Start the app
    if let Some(app) = app.as_mut() {
        app.exec();
    } else {
        eprintln!("Failed to execute QGuiApplication.")
    }
}

async fn download(file:&str, url: &str) -> Result<(), String> {
    let mut response = reqwest::get(url).await.map_err(|_| "reqwest get failed, check url")?;
    let content = response.bytes().await.map_err(|_| "reqwest failed to get, check connection")?;
    let path = Path::new(&file);
    let mut file = tokio::fs::File::create(&path).await.map_err(|_| "Failed to create file")?;
    file.write_all(&content).await.map_err(|_| "Failed to write in file")?;
    Ok(())
}

fn get_json() -> Value {
    let data = match fs::read_to_string("resources/hashmap.json") {
        Ok(f) => f,
        Err(_e) => {
            eprintln!("Cannot read \"resources/hashmap.json\": {}", _e);
            return Value::Null
        }
    };
    serde_json::from_str(&data).expect("Invalid JSON")
}

fn json_check_array(json: &Value, index: &str, len: usize) -> bool {
    if let Some(file_hashes) = json .get(index).and_then(|v| v.as_object()) {
        for (file, values) in file_hashes.iter() {
            if let Some(array) = values.as_array() {
                if array.len() != len {
                    eprintln!("Error in JSON file, in `{}` index, element `{}` is not an array of size of {}", index, file, len);
                    return true
                }
            } else {
                eprintln!("Error in JSON file, in `{}` index, element `{}` is not an array", index, file);
                return true
            }
        }
    }
    false
}
fn json_is_sanitized(json: &Value) -> bool {
    if json_check_array(&json, "core_files", 2) { return true }
    if json_check_array(&json, "mod_files", 3) { return true }
    if json_check_array(&json, "add_ons", 2) { return true }
    false
}

async fn check_resource() -> bool {
    let json = get_json();
    if json != Value::Null {
        if json_is_sanitized(&json) { return true }
        if let Some(file_hashes) = &json.get("resources").and_then(|v| v.as_object()) {
            for (file, url) in file_hashes.iter() {
                if !Path::new(&format!("resources/{}", &file)).exists() {
                    match download(&format!("resources/{}", &file), &url.to_string()).await {
                        Ok(()) => (),
                        Err(_e) => {
                            eprintln!("Download failed: {}", _e);
                            return true
                        }
                    }
                }
            }
        }
    }
    false
}