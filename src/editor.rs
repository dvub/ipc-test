use std::{
    os::unix::process::CommandExt,
    process::{Child, Command},
    thread::spawn,
};

use baseview::{
    Event, EventStatus, Size, Window, WindowHandle, WindowHandler, WindowOpenOptions,
    WindowScalePolicy,
};
use nih_plug::editor::{Editor, ParentWindowHandle};

use crate::{editor, thread};

#[derive(Default)]
pub struct IPCEditor {}

impl IPCEditor {}

struct Handler {}

impl WindowHandler for Handler {
    fn on_frame(&mut self, window: &mut Window) {
        // println!("hi");
    }

    fn on_event(&mut self, window: &mut Window, event: Event) -> EventStatus {
        EventStatus::Ignored
    }
}

unsafe impl Send for Instance {}
struct Instance {
    window_handle: WindowHandle,
    child_handle: Child,
}
impl Drop for Instance {
    fn drop(&mut self) {
        self.window_handle.close();
        self.child_handle.kill().unwrap();
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

        if let ParentWindowHandle::X11Window(id) = parent {
            println!("Parent window handle:{}", id);
            spawn(move || {
                thread::listen_for_client_id(id).unwrap();
            });

            let child_handle =
                Command::new("/home/kaya/projects/audio-dev/ipc-test/target/debug/gui")
                    .spawn()
                    .unwrap();

            let window_handle =
                baseview::Window::open_parented(&parent, options, move |window| Handler {});

            return Box::new(Instance {
                window_handle,
                child_handle,
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
