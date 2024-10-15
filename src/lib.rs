use baseview::{
    Event, EventStatus, Size, Window, WindowHandle, WindowHandler, WindowOpenOptions,
    WindowScalePolicy,
};
use ipc::listen_for_client_id;
use nih_plug::{
    editor::{Editor, ParentWindowHandle},
    prelude::GuiContext,
};
use std::{process::Command, sync::Arc, thread::spawn};
use x11rb::protocol::xproto::reparent_window;

mod ipc;

#[derive(Default)]
pub struct IPCEditor {}

impl IPCEditor {}

struct Handler {}

impl WindowHandler for Handler {
    fn on_frame(&mut self, _window: &mut Window) {
        // println!("hi");
    }

    fn on_event(&mut self, _window: &mut Window, _event: Event) -> EventStatus {
        EventStatus::Ignored
    }
}

unsafe impl Send for Instance {}
struct Instance {
    window: WindowHandle,
    daemon_pid: usize,
}
impl Drop for Instance {
    fn drop(&mut self) {
        self.window.close();
        self.kill_daemon();
    }
}

impl Instance {
    fn kill_daemon(&mut self) {
        let kill_output = Command::new("kill")
            // TODO:
            // could be -15 etc
            .arg("-9")
            .arg(self.daemon_pid.to_string())
            .output()
            .unwrap();

        println!("{}", String::from_utf8(kill_output.stderr).unwrap());
    }
}

impl Editor for IPCEditor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        _context: Arc<dyn GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        let options = WindowOpenOptions {
            scale: WindowScalePolicy::SystemScaleFactor,
            size: Size {
                width: 720.0,
                height: 720.0,
            },
            // TODO:
            // change name to something cool
            title: "Plug-in".to_owned(),
        };

        // TODO:
        // fix this massive if let
        if let ParentWindowHandle::X11Window(embedder_id) = parent {
            // println!("Parent window handle:{}", embedder_id);

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
            let pid = gui::daemon::start_daemon(name).unwrap();

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
        // TODO:
        // make this a field on a struct somewhere
        (720, 720)
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        false
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}

    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}

    fn param_values_changed(&self) {}
}
