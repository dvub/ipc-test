use nih_plug::editor::Editor;

pub struct IPCEditor {}

impl Editor for IPCEditor {
    fn spawn(
        &self,
        _parent: nih_plug::prelude::ParentWindowHandle,
        _context: std::sync::Arc<dyn nih_plug::prelude::GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        gui_lib::run();
        Box::new(())
    }

    fn size(&self) -> (u32, u32) {
        (0, 0)
    }

    fn set_scale_factor(&self, factor: f32) -> bool {
        false
    }

    fn param_value_changed(&self, id: &str, normalized_value: f32) {
        todo!()
    }

    fn param_modulation_changed(&self, id: &str, modulation_offset: f32) {
        todo!()
    }

    fn param_values_changed(&self) {
        todo!()
    }
}
