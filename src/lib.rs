use editor::IPCEditor;
use nih_plug::prelude::*;

use serde::Serialize;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

mod editor;
mod thread;

#[derive(Params)]
struct PluginParams {
    #[id = "gain"]
    gain: FloatParam,
}
#[derive(Serialize)]
struct SerializableParams {
    gain: f32,
}

impl Default for PluginParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new("Gain", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
        }
    }
}

struct IPCPlugin {
    params: Arc<PluginParams>,
    should_cancel_thread: Arc<AtomicBool>,
}

impl Default for IPCPlugin {
    fn default() -> Self {
        let params = Arc::new(PluginParams::default());

        Self {
            params,
            should_cancel_thread: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Plugin for IPCPlugin {
    const NAME: &'static str = "IPC TEST";
    const VENDOR: &'static str = "DVUB";
    // You can use `env!("CARGO_PKG_HOMEPAGE")` to reference the homepage field from the
    // `Cargo.toml` file here
    const URL: &'static str = "TODO";
    const EMAIL: &'static str = "TODO";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),

            aux_input_ports: &[],
            aux_output_ports: &[],

            // Individual ports and the layout as a whole can be named here. By default these names
            // are generated as needed. This layout will be called 'Stereo', while the other one is
            // given the name 'Mono' based no the number of input and output channels.
            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    // Setting this to `true` will tell the wrapper to split the buffer up into smaller blocks
    // whenever there are inter-buffer parameter changes. This way no changes to the plugin are
    // required to support sample accurate automation and the wrapper handles all of the boring
    // stuff like making sure transport and other timing information stays consistent between the
    // splits.
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        nih_log!("Initialized!");

        true
    }

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    // this is a lie kind of
    type BackgroundTask = ();

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        Some(Box::new(IPCEditor::default()))
    }
    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // if we wanted to do process samples individually,
        // it would be here

        for channel_samples in buffer.iter_samples() {
            for sample in channel_samples {
                *sample *= self.params.gain.value();
            }
        }

        ProcessStatus::Normal
    }

    // This can be used for cleaning up special resources like socket connections whenever the
    // plugin is deactivated. Most plugins won't need to do anything here.
    fn deactivate(&mut self) {
        self.should_cancel_thread.store(true, Ordering::Relaxed);
        nih_log!("Plugin has been deactivated. ");
    }
}

impl ClapPlugin for IPCPlugin {
    const CLAP_ID: &'static str = "com.moist-plugins-gmbh.gain";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A smoothed gain parameter example plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for IPCPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"IPCTESTDVUB12345";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(IPCPlugin);
nih_export_vst3!(IPCPlugin);
