use interprocess::local_socket::{prelude::*, Name, Stream};
use raw_window_handle::HasRawWindowHandle;
use std::io::prelude::*;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

pub mod daemon;

pub fn run(name: Name) -> std::io::Result<()> {
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
        send_id(name, id_u32);
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

fn send_id(name: Name, id: u32) {
    let mut conn = Stream::connect(name).expect("couldnt connect to socket");

    // --- 1. WRITE (OUT) ---
    conn.write_all(&id.to_be_bytes())
        .expect("Failed to send ping");
    conn.write_all(b"\n").expect("Failed to send ping");
}
