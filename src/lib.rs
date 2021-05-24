extern crate ladspa;
extern crate nnnoiseless;

use ladspa::{PluginDescriptor, PortDescriptor, Port, DefaultValue, Data, Plugin, PortConnection};
use std::default::Default;

struct Denoiser {
    denoiser: Box<nnnoiseless::DenoiseState<'static>>,
    first: bool,
    last_sample: [f32; nnnoiseless::DenoiseState::FRAME_SIZE],
}

fn new_denoiser(_: &PluginDescriptor, _sample_rate: u64) -> Box<Plugin + Send> {
    Box::new(Denoiser {
        denoiser: nnnoiseless::DenoiseState::new(),
        first: true,
        last_sample: [0.0f32; nnnoiseless::DenoiseState::FRAME_SIZE],
    })
}


impl Plugin for Denoiser {
    fn activate(&mut self) {
    }

    fn run<'a>(&mut self, sample_count: usize, ports: &[&'a PortConnection<'a>]) {
        let input = ports[0].unwrap_audio();
        let mut output = ports[1].unwrap_audio_mut();
        let mut out_buf = [0.0; nnnoiseless::DenoiseState::FRAME_SIZE];
        let mut input_buf = [0.0f32; nnnoiseless::DenoiseState::FRAME_SIZE];
        input_buf[(nnnoiseless::DenoiseState::FRAME_SIZE - input.len())..].copy_from_slice(&input);
        if !self.first {
            input_buf[..(nnnoiseless::DenoiseState::FRAME_SIZE - input.len())].copy_from_slice(&self.last_sample[input.len()..]);
        }
        self.last_sample = input_buf;
        input_buf.iter_mut().for_each(|sample| *sample = *sample * 32768.0f32);
        for chunk in input_buf.chunks_exact(nnnoiseless::DenoiseState::FRAME_SIZE) {
            if !self.first {
                self.denoiser.process_frame(&mut out_buf[..], chunk);
            }
            self.first = false;
        }
        out_buf.iter_mut().for_each(|sample| *sample = *sample / 32768.0f32);
        for i in 0..sample_count {
            output[i] = out_buf[i];
        }
    }
}

#[no_mangle]
pub fn get_ladspa_descriptor(index: u64) -> Option<PluginDescriptor> {
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
