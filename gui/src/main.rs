use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event_loop::EventLoop,
    rwh_06::{HasWindowHandle, XlibWindowHandle},
    window::WindowBuilder,
};
use wry::{Rect, WebViewBuilder};

pub fn main() {
    let window_size = LogicalSize::new(720, 720);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(window_size)
        .build(&event_loop)
        .unwrap();

    let raw = window.id();

    // #[cfg(not(target_os = "linux"))]
    let builder = WebViewBuilder::new_as_child(&window);

    // #[cfg(target_os = "linux")]
    // let builder = WebViewBuilder::new_gtk(&gtk_fixed);

    let _webview = builder
        .with_url("https://tauri.app")
        .with_bounds(Rect {
            position: LogicalPosition::new(0, 0).into(),
            size: window_size.into(),
        })
        .build()
        .unwrap();

    event_loop.run(|_, _, _| {});
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
