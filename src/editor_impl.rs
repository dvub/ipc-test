use crate::{
    gui::daemon,
    instance::Instance,
    ipc::{self, listen_for_client_id},
    IPCEditor,
};

use baseview::{Event, EventStatus, MouseEvent, Size, WindowOpenOptions, WindowScalePolicy};

use keyboard_types::KeyboardEvent;
use nih_plug::{
    editor::{Editor, ParentWindowHandle},
    prelude::GuiContext,
};

use std::{
    sync::{atomic::Ordering, Arc},
    thread::spawn,
};

use x11rb::{
    protocol::xproto::{reparent_window, set_input_focus, InputFocus},
    rust_connection::RustConnection,
    CURRENT_TIME,
};
type KeyboardHandler = dyn Fn(KeyboardEvent) -> bool + Send + Sync;
type MouseHandler = dyn Fn(MouseEvent) -> EventStatus + Send + Sync;

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
            // - improve error handling here

            let window_handle = baseview::Window::open_parented(&parent, options, move |_| {
                let (x_conn, _screen_num) = x11rb::connect(None).unwrap();

                // should we unwrap this?
                let reparent_request =
                    reparent_window(&x_conn, client_id, embedder_id, 0, 0).unwrap();

                // we probably want to unwrap or expect this
                // if this request is fucked up for some reason
                // we cannot let things continue normally
                reparent_request.check().unwrap();

                Handler {
                    x_conn,
                    client_id,
                    wants_initial_focus: true,
                    keyboard_handler: self.keyboard_handler.clone(),
                    mouse_handler: self.mouse_handler.clone(),
                }
            });

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
    wants_initial_focus: bool,
    x_conn: RustConnection,
    client_id: u32,
    keyboard_handler: Arc<KeyboardHandler>,
    mouse_handler: Arc<MouseHandler>,
}

impl baseview::WindowHandler for Handler {
    fn on_frame(&mut self, _window: &mut baseview::Window) {
        if self.wants_initial_focus {
            let focus_request =
                set_input_focus(&self.x_conn, InputFocus::NONE, self.client_id, CURRENT_TIME)
                    .unwrap();

            // TODO:
            // should we unwrap, and potentially panic on this?
            focus_request.check().unwrap();
            self.wants_initial_focus = false;
        }
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
