use interprocess::local_socket::{prelude::*, GenericFilePath, GenericNamespaced, Stream};
use std::io::prelude::*;
use std::{sync::Mutex, thread};

use serde_json::Value;
use tauri::{AppHandle, Manager, State, WebviewWindowBuilder};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn get_gain(state: State<Mutex<Params>>) -> f32 {
    state.lock().unwrap().gain
}

struct Params {
    gain: f32,
}

fn spawn_ipc_listener(app_handle: AppHandle) {
    thread::spawn(move || {
        let name = if GenericNamespaced::is_supported() {
            "example.sock".to_ns_name::<GenericNamespaced>().unwrap()
        } else {
            "/tmp/example.sock".to_fs_name::<GenericFilePath>().unwrap()
        };
        let mut conn = Stream::connect(name).unwrap();
        let mut buffer = [0; 128];

        loop {
            conn.write_all(b"ping").expect("Failed to send ping");
            let bytes_read = conn.read(&mut buffer).expect("Failed to read from socket");

            let message = String::from_utf8((buffer[..bytes_read]).to_vec()).unwrap();
            let v: Value = serde_json::from_str(&message).unwrap();

            let new_gain = v["gain"].as_f64().unwrap() as f32;
            // println!("Client received: {}", new_gain);
            let mutex = app_handle.state::<Mutex<Params>>();
            let mut lock = mutex.lock().unwrap();
            lock.gain = new_gain;
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_gain])
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(Mutex::new(Params { gain: 0.0 }));

            let handle = app.handle();
            spawn_ipc_listener(handle.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
