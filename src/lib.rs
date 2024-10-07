use nih_plug::prelude::*;

use std::{sync::Arc, thread::spawn};

#[derive(Params)]
struct PluginParams {
    #[id = "gain"]
    gain: FloatParam,
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
}

impl Default for IPCPlugin {
    fn default() -> Self {
        let params = Arc::new(PluginParams::default());

        Self { params }
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
    type BackgroundTask = ();

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let params_clone = self.params.clone();
        // a thread appears!
        spawn(move || {
            use interprocess::local_socket::{
                prelude::*, GenericNamespaced, ListenerOptions, Stream,
            };
            use std::io::{self, prelude::*, BufReader};

            // Define a function that checks for errors in incoming connections. We'll use this to filter
            // through connections that fail on initialization for one reason or another.
            fn handle_error(conn: io::Result<Stream>) -> Option<Stream> {
                match conn {
                    Ok(c) => Some(c),
                    Err(e) => {
                        eprintln!("Incoming connection failed: {e}");
                        None
                    }
                }
            }

            // Pick a name.
            let printname = "example.sock";
            let name = printname.to_ns_name::<GenericNamespaced>()?;

            // Configure our listener...
            let opts = ListenerOptions::new().name(name);

            // ...then create it.
            let listener = match opts.create_sync() {
                Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
                    // When a program that uses a file-type socket name terminates its socket server
                    // without deleting the file, a "corpse socket" remains, which can neither be
                    // connected to nor reused by a new listener. Normally, Interprocess takes care of
                    // this on affected platforms by deleting the socket file when the listener is
                    // dropped. (This is vulnerable to all sorts of races and thus can be disabled.)
                    //
                    // There are multiple ways this error can be handled, if it occurs, but when the
                    // listener only comes from Interprocess, it can be assumed that its previous instance
                    // either has crashed or simply hasn't exited yet. In this example, we leave cleanup
                    // up to the user, but in a real application, you usually don't want to do that.
                    eprintln!(
                        "Error: could not start server because the socket file is occupied. Please check if
                        {printname} is in use by another process and try again."
                    );
                    return Err(e);
                }
                x => x?,
            };

            // The syncronization between the server and client, if any is used, goes here.
            eprintln!("Server running at {printname}");

            // Preemptively allocate a sizeable buffer for receiving at a later moment. This size should
            // be enough and should be easy to find for the allocator. Since we only have one concurrent
            // client, there's no need to reallocate the buffer repeatedly.
            let mut buffer = String::with_capacity(128);
            if let Some(mut conn) = listener.incoming().filter_map(handle_error).next() {
                loop {
                    println!("FML");
                    let mut buffer = [0; 4]; // Buffer to read messages
                    let bytes_read = conn.read(&mut buffer).expect("Failed to read from socket");

                    let message = String::from_utf8_lossy(&buffer[..bytes_read]);
                    println!("Server received: {}", message);

                    let reply = if message.trim() == "ping" {
                        "pong"
                    } else {
                        "ping"
                    };
                    conn.write_all(reply.as_bytes())
                        .expect("Failed to write to socket");
                    println!("BANG");
                }
            }

            Ok(())
        });
        true
    }

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
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
    fn deactivate(&mut self) {}
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
