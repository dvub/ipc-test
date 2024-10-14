use daemonize::Daemonize;
use interprocess::local_socket::{prelude::*, GenericNamespaced, Stream};
use raw_window_handle::HasRawWindowHandle;
use std::fs::File;
use std::{fs::read_to_string, io::prelude::*};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tempdir::TempDir;
use wry::WebViewBuilder;

pub fn run() -> std::io::Result<()> {
    let window_size = LogicalSize::new(720, 720);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(window_size)
        .build(&event_loop)
        .unwrap();

    #[cfg(any(target_os = "windows", target_os = "macos",))]
    let builder = WebViewBuilder::new(&window);

    #[cfg(not(any(target_os = "windows", target_os = "macos",)))]
    let builder = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        let vbox = window.default_vbox().unwrap();
        WebViewBuilder::new_gtk(vbox)
    };

    let _webview = builder
        .with_url("http://tauri.app")
        .with_drag_drop_handler(|e| {
            match e {
                wry::DragDropEvent::Enter { paths, position } => {
                    println!("DragEnter: {position:?} {paths:?} ")
                }
                wry::DragDropEvent::Over { position } => println!("DragOver: {position:?} "),
                wry::DragDropEvent::Drop { paths, position } => {
                    println!("DragDrop: {position:?} {paths:?} ")
                }
                wry::DragDropEvent::Leave => println!("DragLeave"),
                _ => {}
            }

            true
        })
        .build()
        .expect("build failed..");

    let raw_handle = window.raw_window_handle();
    if let tao::rwh_05::RawWindowHandle::Xlib(xlib_handle) = raw_handle {
        let id_u32 = xlib_handle.window as u32;
        send_id(id_u32);
    }

    println!("CLIENT: beginning event loop.");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit
        }
    });
}

fn send_id(id: u32) {
    let printname = "example.sock";
    let name = printname.to_ns_name::<GenericNamespaced>().unwrap();
    let mut conn = Stream::connect(name).expect("couldnt connect to socket");

    // --- 1. WRITE (OUT) ---
    conn.write_all(&id.to_be_bytes())
        .expect("Failed to send ping");
    conn.write_all(b"\n").expect("Failed to send ping");
}

pub fn start_daemon() -> usize {
    let directory = TempDir::new("IPC_TEST").unwrap();
    let pid_path = directory.path().join("test.pid");

    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file(&pid_path) // Every method except `new` and `start`
        .stdout(stdout) // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr); // Redirect stderr to `/tmp/daemon.err`.

    match daemonize.execute() {
        daemonize::Outcome::Parent(_p) => {}
        daemonize::Outcome::Child(_) => {
            let _ = run();
        }
    };

    // immediately after starting the daemon, retreive the PID from the file
    let pid = read_to_string(pid_path)
        .unwrap()
        .trim()
        .parse::<usize>()
        .unwrap();

    // todo?
    // drop(tmp_file);
    // tmp_dir.close()?;

    pid
}
