use std::{os::unix::process::CommandExt, process::Command};

use baseview::{
    Event, EventStatus, Size, Window, WindowHandler, WindowOpenOptions, WindowScalePolicy,
};
use nih_plug::editor::{Editor, ParentWindowHandle};

use crate::{editor, thread};

#[derive(Default)]
pub struct IPCEditor {}

impl IPCEditor {}

struct Re {}
unsafe impl Send for Re {}

impl WindowHandler for Re {
    fn on_frame(&mut self, window: &mut Window) {
        // println!("hi");
    }

    fn on_event(&mut self, window: &mut Window, event: Event) -> EventStatus {
        EventStatus::Ignored
    }
}

struct Instance {}

impl Editor for IPCEditor {
    fn spawn(
        &self,
        parent: nih_plug::prelude::ParentWindowHandle,
        _context: std::sync::Arc<dyn nih_plug::prelude::GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        let options = WindowOpenOptions {
            scale: WindowScalePolicy::SystemScaleFactor,
            size: Size {
                width: 200.0,
                height: 200.0,
            },
            title: "Plug-in".to_owned(),
        };

        if let ParentWindowHandle::X11Window(id) = parent {
            println!("Parent window handle:{}", id);
            thread::ipc_server_listener(id);
        }
        let handle = baseview::Window::open_parented(&parent, options, move |window| Re {});

        // TODO:
        // make cross platform

        Box::new(Instance {})
    }

    fn size(&self) -> (u32, u32) {
        (100, 100)
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        false
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}

    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}

    fn param_values_changed(&self) {}
}
