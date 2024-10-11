use std::{os::unix::process::CommandExt, process::Command};

use nih_plug::editor::{Editor, ParentWindowHandle};

use crate::{editor, thread};

#[derive(Default)]
pub struct IPCEditor {}

impl IPCEditor {}

impl Editor for IPCEditor {
    fn spawn(
        &self,
        parent: nih_plug::prelude::ParentWindowHandle,
        _context: std::sync::Arc<dyn nih_plug::prelude::GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        if let ParentWindowHandle::X11Window(id) = parent {
            println!("Parent window handle:{}", id);
            thread::ipc_server_listener(id);
        }
        // TODO:
        // make cross platform

        Box::new(())
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
