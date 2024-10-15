use crate::{
    gui::daemon,
    instance::Instance,
    ipc::{self, listen_for_client_id},
    EventLoopHandler, IPCEditor, KeyboardHandler, MouseHandler,
};

use baseview::{Event, EventStatus};
use keyboard_types::KeyboardEvent;

use nih_plug::{
    editor::{Editor, ParentWindowHandle},
    prelude::{GuiContext, ParamSetter},
};
use serde_json::Value;
use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    thread::spawn,
};
use wry::WebView;
use x11rb::protocol::xproto::reparent_window;

impl Editor for IPCEditor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        _context: Arc<dyn GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        let options = WindowOpenOptions {
            scale: WindowScalePolicy::SystemScaleFactor,
            size: Size {
                width: self.width.load(Ordering::Relaxed) as f64,
                height: self.height.load(Ordering::Relaxed) as f64,
            },
            // TODO:
            // change name to something cool
            title: "Plug-in".to_owned(),
        };

        // TODO:
        // fix this massive if let
        if let ParentWindowHandle::X11Window(embedder_id) = parent {
            let window_handle =
                baseview::Window::open_parented(&parent, options, move |_| Handler {});

            // start IPC server
            // name can be whatever
            let name = ipc::get_open_socket_name("IPC_TEST__").unwrap();

            // clone the name, and move it into a new thread
            let name_clone = name.clone();
            let handle = spawn(move || listen_for_client_id(name_clone).unwrap());

            // start GUI, which communicates with IPC server
            // TODO: can this ever try connecting to the server *before* the server is open?
            let pid = daemon::start_daemon(name, self).unwrap();

            // wait until we get some response from our IPC server

            // TODO:
            // if something happens where the GUI doesn't open for whatever reason,
            // this will totally block all other execution
            // and that is really problematic
            let client_id = handle.join().unwrap();

            // x11 stuff
            // TODO:
            // - should we store this x11 connection for later?
            // - improve error handling here
            let (x_conn, _screen_num) = x11rb::connect(None).unwrap();

            let c = reparent_window(&x_conn, client_id, embedder_id, 0, 0).unwrap();
            c.check().unwrap();

            return Box::new(Instance {
                window: window_handle,
                daemon_pid: pid,
            });
        }

        Box::new(())
    }

    fn size(&self) -> (u32, u32) {
        (
            self.width.load(Ordering::Relaxed),
            self.height.load(Ordering::Relaxed),
        )
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        // TODO: implement for Windows and Linux
        false
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}

    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}

    fn param_values_changed(&self) {}
}

pub struct Handler {
    context: Arc<dyn GuiContext>,
    event_loop_handler: Arc<EventLoopHandler>,
    keyboard_handler: Arc<KeyboardHandler>,
    mouse_handler: Arc<MouseHandler>,
    webview: WebView,
    // events_receiver: Receiver<Value>,
    pub width: Arc<AtomicU32>,
    pub height: Arc<AtomicU32>,
}

impl Handler {
    /*
    pub fn resize(&self, window: &mut baseview::Window, width: u32, height: u32) {
        self.webview.set_bounds(wry::Rect {
            x: 0,
            y: 0,
            width,
            height,
        });
        self.width.store(width, Ordering::Relaxed);
        self.height.store(height, Ordering::Relaxed);
        self.context.request_resize();
        window.resize(Size {
            width: width as f64,
            height: height as f64,
        });
    }*/

    pub fn send_json(&self, json: Value) {
        let json_str = json.to_string();
        let json_str_quoted =
            serde_json::to_string(&json_str).expect("Should not fail: the value is always string");
        self.webview
            .evaluate_script(&format!("onPluginMessageInternal({});", json_str_quoted))
            .unwrap();
    }

    /*
    pub fn next_event(&self) -> Result<Value, crossbeam::channel::TryRecvError> {
        self.events_receiver.try_recv()
    }
    */
}

impl baseview::WindowHandler for Handler {
    fn on_frame(&mut self, window: &mut baseview::Window) {
        let setter = ParamSetter::new(&*self.context);
        (self.event_loop_handler)(self, setter, window);
    }

    fn on_event(&mut self, _window: &mut baseview::Window, event: Event) -> EventStatus {
        match event {
            Event::Keyboard(event) => {
                if (self.keyboard_handler)(event) {
                    EventStatus::Captured
                } else {
                    EventStatus::Ignored
                }
            }
            Event::Mouse(mouse_event) => (self.mouse_handler)(mouse_event),
            _ => EventStatus::Ignored,
        }
    }
}
