use std::{
    process::{Child, Command},
    thread::spawn,
};

use baseview::{
    Event, EventStatus, Size, Window, WindowHandle, WindowHandler, WindowOpenOptions,
    WindowScalePolicy,
};
use nih_plug::editor::{Editor, ParentWindowHandle};
use x11rb::protocol::xproto::reparent_window;

use crate::thread::get_client_id;

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
    window_handle: WindowHandle,
    daemon_path: Child,
}
impl Drop for Instance {
    fn drop(&mut self) {
        // close process
        self.window_handle.close();

        self.daemon_path.kill().unwrap()
    }
}

impl Editor for IPCEditor {
    fn spawn(
        &self,
        parent: nih_plug::prelude::ParentWindowHandle,
        _context: std::sync::Arc<dyn nih_plug::prelude::GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        let options = WindowOpenOptions {
            scale: WindowScalePolicy::SystemScaleFactor,
            size: Size {
                width: 720.0,
                height: 720.0,
            },
            title: "Plug-in".to_owned(),
        };

        // TODO:
        // fix this massive if let
        if let ParentWindowHandle::X11Window(embedder_id) = parent {
            // println!("Parent window handle:{}", embedder_id);

            let window_handle =
                baseview::Window::open_parented(&parent, options, move |_| Handler {});

            // start IPC server
            let handle = spawn(move || get_client_id().unwrap());
            // start GUI, which communicates with IPC server
            let h = Command::new("/run/media/kaya/Media/projects/rust/ipc-test/target/debug/gui")
                .spawn()
                .unwrap();
            // wait until we get some response from our IPC server
            let client_id = handle.join().unwrap();

            // x11 stuff
            // TODO:
            // - should we store this x11 connection for later?
            // - improve error handling here
            let (x_conn, _screen_num) = x11rb::connect(None).unwrap();
            let c = reparent_window(&x_conn, client_id, embedder_id, 0, 0).unwrap();
            c.check().unwrap();

            return Box::new(Instance {
                window_handle,
                daemon_path: h,
            });
        }
        Box::new(())
    }

    fn size(&self) -> (u32, u32) {
        (720, 720)
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        false
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}

    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}

    fn param_values_changed(&self) {}
}
