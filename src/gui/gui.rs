use interprocess::local_socket::{prelude::*, Name, Stream};

use raw_window_handle::HasRawWindowHandle;
use std::io::{self, prelude::*};
use std::sync::atomic::Ordering;
use tao::dpi::LogicalPosition;
use tao::event::{Event, WindowEvent};

use tao::platform::unix::WindowExtUnix;

use tao::{
    dpi::LogicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::http::Request;
use wry::{Rect, WebViewBuilder, WebViewBuilderExtUnix};
use x11rb::protocol::xproto::{set_input_focus, InputFocus};
use x11rb::CURRENT_TIME;

use crate::{HTMLSource, IPCEditor};

pub fn run(name: Name, editor: &IPCEditor) -> io::Result<()> {
    let (x_conn, _screen_num) = x11rb::connect(None).unwrap();

    // set_var("WINIT_UNIX_BACKEND", "x11");

    let width = editor.width.clone();
    let height = editor.height.clone();
    let developer_mode = editor.developer_mode;
    let source = editor.source.clone();
    let background_color = editor.background_color;
    let custom_protocol = editor.custom_protocol.clone();
    let _event_loop_handler = editor.event_loop_handler.clone();
    let keyboard_handler = editor.keyboard_handler.clone();
    let _mouse_handler = editor.mouse_handler.clone();

    let window_size = LogicalSize::new(
        width.load(Ordering::Relaxed),
        height.load(Ordering::Relaxed),
    );

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(window_size)
        .build(&event_loop)
        .unwrap();

    let mut client_id = 0;
    let raw_handle = window.raw_window_handle();
    if let tao::rwh_05::RawWindowHandle::Xlib(xlib_handle) = raw_handle {
        client_id = xlib_handle.window as u32;
    }

    #[cfg(any(target_os = "windows", target_os = "macos",))]
    let builder = WebViewBuilder::new(&window);

    #[cfg(not(any(target_os = "windows", target_os = "macos",)))]
    let mut builder = {
        let vbox = window.default_vbox().unwrap();
        WebViewBuilder::new_gtk(vbox)
    };

    builder = builder
        .with_bounds(Rect {
            // why would this be anything other than 0,0?
            position: LogicalPosition::new(0, 0).into(),
            size: window_size.into(),
        })
        .with_initialization_script(include_str!("script.js"))
        .with_accept_first_mouse(true)
        .with_devtools(developer_mode)
        .with_focused(false)
        .with_ipc_handler(move |msg: Request<String>| {
            let body = msg.body();
            if body == "FOCUS_IN" {
                let focus_request =
                    set_input_focus(&x_conn, InputFocus::NONE, client_id, CURRENT_TIME)
                        .expect("oh no!");
                focus_request.check().expect("FUCK");
            }
        })
        // TODO!!!!
        /*  .with_web_context(&mut web_context)
         .with_initialization_script(include_str!("script.js"))
        .with_ipc_handler(move |msg: String| {
            if let Ok(json_value) = serde_json::from_str(&msg) {
                let _ = events_sender.send(json_value);
            } else {
                panic!("Invalid JSON from web view: {}.", msg);
            }
        })
        */
        .with_background_color(background_color);

    if let Some(custom_protocol) = custom_protocol.as_ref() {
        let handler = custom_protocol.1.clone();
        builder = builder.with_custom_protocol(custom_protocol.0.to_owned(), move |request| {
            handler(&request).unwrap()
        });
    }

    builder = match source.as_ref() {
        HTMLSource::String(html_str) => builder.with_html(*html_str),
        HTMLSource::URL(url) => builder.with_url(*url),
    };

    // TODO:
    // should probably do something with this
    let _webview = builder.build().expect("build failed..");

    // important!!

    send_id(name, client_id)?;

    println!("CLIENT: beginning event loop.");

    // we are now running the event loop
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                WindowEvent::KeyboardInput { event, .. } => keyboard_handler(event),
                // TODO:
                // mouse handling

                // another todo:
                // why are we working with bools?
                _ => false,
            },

            _ => false,
        };
        *control_flow = ControlFlow::Wait;
    });
}

fn send_id(name: Name, id: u32) -> io::Result<()> {
    let mut conn = Stream::connect(name)?;

    // --- 1. WRITE (OUT) ---
    conn.write_all(&id.to_be_bytes())
        .expect("Failed to send ping");
    conn.write_all(b"\n")?;

    Ok(())
}
