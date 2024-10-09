use gtk::prelude::BoxExt;
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event_loop::EventLoop,
    platform::unix::WindowExtUnix,
    window::WindowBuilder,
};
use wry::{Rect, WebViewBuilder, WebViewBuilderExtUnix, WebViewExtUnix};

pub fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(not(target_os = "linux"))]
    let builder = WebViewBuilder::new_as_child(&window);

    #[cfg(target_os = "linux")]
    let gtk_fixed = {
        let vbox = window.default_vbox().unwrap(); // tao adds a gtk::Box by default
        let fixed = gtk::Fixed::new();
        vbox.pack_start(&fixed, true, true, 0);
        fixed
    };

    #[cfg(target_os = "linux")]
    let builder = WebViewBuilder::new_gtk(&gtk_fixed);

    let webview = builder
        .with_url("https://tauri.app")
        .with_bounds(Rect {
            position: LogicalPosition::new(100, 100).into(),
            size: LogicalSize::new(200, 200).into(),
        })
        .build()
        .unwrap();

    loop {}
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
