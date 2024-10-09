use gui::run;

pub fn main() {
    run();
}

/*

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
} */
