use ladspa::{PluginDescriptor, PortDescriptor, Port, Plugin, PortConnection};

const FRAME_SIZE: usize = nnnoiseless::DenoiseState::FRAME_SIZE;

struct Denoiser {
    denoiser: Box<nnnoiseless::DenoiseState<'static>>,
    first: bool,
    last_sample: [f32; FRAME_SIZE],
}

fn new_denoiser(_: &PluginDescriptor, _sample_rate: u64) -> Box<dyn Plugin + Send> {
    Box::new(Denoiser {
        denoiser: nnnoiseless::DenoiseState::new(),
        first: true,
        last_sample: [0.0f32; FRAME_SIZE],
    })
}


impl Plugin for Denoiser {
    fn activate(&mut self) {
    }

    fn run<'a>(&mut self, _sample_count: usize, ports: &[&'a PortConnection<'a>]) {
        let input = ports[0].unwrap_audio();
        let mut output = ports[1].unwrap_audio_mut();
        let mut output = output.iter_mut();
        let mut output_buf = [0.0f32; FRAME_SIZE];

        for chunk in input.chunks(FRAME_SIZE) {
            let mut input_buf = [0.0f32; FRAME_SIZE];
            input_buf[FRAME_SIZE.saturating_sub(chunk.len())..].copy_from_slice(&chunk);
            if !self.first {
                input_buf[..FRAME_SIZE.saturating_sub(chunk.len())].copy_from_slice(&self.last_sample[chunk.len()..]);
            }
            self.last_sample = input_buf;
            input_buf.iter_mut().for_each(|sample| *sample = *sample * 32768.0f32);
            self.denoiser.process_frame(&mut output_buf, &input_buf);
            self.first = false;

            for (output, &sample) in output.by_ref().zip(output_buf.iter()) {
                if !self.first {
                    *output = sample / 32768.0f32;
                } else {
                    *output = 0.0f32;
                }
            }
        }
    }
}

#[no_mangle]
pub extern "Rust" fn get_ladspa_descriptor(index: u64) -> Option<PluginDescriptor> {
    match index {
        0 => {
            Some(PluginDescriptor {
                unique_id: 400,
                label: "mono_nnnoiseless",
                properties: ladspa::PROP_NONE,
                name: "Mono NNNoiseless",
                maker: "kat witch",
                copyright: "None",
                ports: vec![
                    Port {
                        name: "Audio In",
                        desc: PortDescriptor::AudioInput,
                        ..Default::default()
                    },
                    Port {
                        name: "Audio Out",
                        desc: PortDescriptor::AudioOutput,
                        ..Default::default()
                    },
                ],
                new: new_denoiser,

            })
        },
        _ => None
    }
}
