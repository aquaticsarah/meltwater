// Meltwater: VST2 API wrapper
// Copyright 2021, Sarah Ocean and the Meltwater project contributors.
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Arc, Mutex, MutexGuard};

use vst::api::{Events, Supported};
use vst::buffer::AudioBuffer;
use vst::plugin::{
  CanDo,
  Category,
  HostCallback,
  Info,
  Plugin,
  PluginParameters
};

use crate::opus_codec::OpusCodec;


const NUM_PARAMETERS: usize = 1;

struct MeltwaterPluginParamsInner {
  raw_params: [f32; NUM_PARAMETERS],
  bitrate_kbps: f32,
  opus_codec: OpusCodec,
}

struct ParameterDescriptor {
  name: fn(&MeltwaterPluginParamsInner) -> String,
  format: fn(&MeltwaterPluginParamsInner, f32) -> String,
  unit: fn(&MeltwaterPluginParamsInner, f32) -> String,
  apply: fn(&mut MeltwaterPluginParamsInner, f32),
}

const PARAMETERS: [ParameterDescriptor; NUM_PARAMETERS] = [
  ParameterDescriptor {
    name: |params| {
      "Quality".to_string()
    },
    format: |params, raw_value| {
      format!("{:3.0}%", params.raw_params[0] * 100.0)
    },
    unit: |params, raw_value| {
      "".to_string()
    },
    apply: |params, raw_value| {
      // The bitrate range is set based on two considerations:
      // * At the low end, rates <20kbps degrade too much, including generating
      //   an unwanted 400Hz tone
      // * At the high end, the codec is near-transparent at around 160kbps
      //   in listening tests with 20ms packets. We use 2.5ms packets instead,
      //   which technically reduces the quality a little, but it seems to be
      //   a pretty minimal difference, so we stick with 160kbps as the upper end.
      //
      // From experimentation, an exponential control over this range seems to
      // match the (subjective) quality degradation reasonably well.
      let bitrate_kbps = 20.0 * f32::powf(8.0, raw_value);
      params.bitrate_kbps = bitrate_kbps;
      params.opus_codec.set_bitrate(bitrate_kbps);
    },
  },
];

// Hard-coded default parameter values
// Note that the values here are VST parameter values, so should be
// in the range of [0, 1], with comments specifying the processed
// values
const DEFAULT_PARAMETERS: [f32; NUM_PARAMETERS] = [
  1.0, // Transparent by default
];


struct MeltwaterPluginParams(Mutex<MeltwaterPluginParamsInner>);

impl MeltwaterPluginParams {
  fn new() -> Self {
    let mut inner = MeltwaterPluginParamsInner {
      raw_params: DEFAULT_PARAMETERS.clone(),
      bitrate_kbps: 0.0,
      opus_codec: OpusCodec::new(),
    };

    // Fill out processed parameter values
    for i in 0..NUM_PARAMETERS {
      let value = inner.raw_params[i];
      (PARAMETERS[i].apply)(&mut inner, value);
    }

    Self(Mutex::new(inner))
  }
}

impl MeltwaterPluginParams {
  fn lock(&self) -> MutexGuard<MeltwaterPluginParamsInner> {
    // If another thread (probably the UI) has panicked while holding this lock,
    // then the inner state cannot be trusted.
    // The only options we have are:
    // i) Abort, which is easy but may have undesired effects (some host DAWs will crash)
    // ii) Tear down and rebuild the plugin state, which is nontrivial
    // So we choose option (i) for now
    self.0.lock().unwrap()
  }
}

impl PluginParameters for MeltwaterPluginParams {
  // TODO: Implement presets?
  // TODO: string_to_parameter()

  fn get_parameter_name(&self, index: i32) -> String {
    let inner = self.lock();
    (PARAMETERS[index as usize].name)(&inner)
  }

  fn get_parameter_text(&self, index: i32) -> String {
    let inner = self.lock();
    let raw_value = inner.raw_params[index as usize];
    (PARAMETERS[index as usize].format)(&inner, raw_value)
  }

  fn get_parameter_label(&self, index: i32) -> String {
    let inner = self.lock();
    let raw_value = inner.raw_params[index as usize];
    (PARAMETERS[index as usize].unit)(&inner, raw_value)
  }

  fn get_parameter(&self, index: i32) -> f32 {
    let inner = self.lock();
    inner.raw_params[index as usize]
  }

  fn set_parameter(&self, index: i32, raw_value: f32) {
    let mut inner = self.lock();
    inner.raw_params[index as usize] = raw_value;
    (PARAMETERS[index as usize].apply)(&mut inner, raw_value);
  }

  fn can_be_automated(&self, index: i32) -> bool {
    true
  }
}


struct MeltwaterPlugin {
  #[allow(dead_code)]
  host: HostCallback,

  params: Arc<MeltwaterPluginParams>,
}

impl Plugin for MeltwaterPlugin {
  fn new(host: HostCallback) -> Self {
    Self {
      host: host,
      params: Arc::new(MeltwaterPluginParams::new()),
    }
  }

  fn init(&mut self) {
    // TODO
  }

  fn set_sample_rate(&mut self, rate: f32) {
    // TODO: check we're in the "suspended" state
    // TODO: queue up request until `resume` is called
    // TODO: Set up resampling if rate != 48kHz
    if rate != 48000.0 {
      todo!("Sample rates other than 48kHz are not supported yet");
    }
  }

  fn set_block_size(&mut self, size: i64) {
    // TODO: check we're in the "suspended" state
    // TODO: queue up request until `resume` is called
    // TODO: Use the given value to select the size of various internal buffers
  }

  fn resume(&mut self) {
    // TODO: track "suspended" state
    // TODO: apply changes which were made in the "suspended" state
  }

  fn suspend(&mut self) {
    // TODO: track "suspended" state
  }

  fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
    self.params.clone()
  }

  //fn get_editor(&mut self) -> Option<Box<dyn Editor>>; // TODO


  fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
    // TODO: check we're in the "running" state

    let num_samples = buffer.samples();
    let (audio_in, mut audio_out) = buffer.split();
    let (left_in, right_in) = (audio_in.get(0), audio_in.get(1));
    let (left_out, right_out) = (audio_out.get_mut(0), audio_out.get_mut(1));

    let mut params = self.params.lock();

    params.opus_codec.process_samples(left_in, right_in, left_out, right_out);
  }

  fn process_events(&mut self, events: &Events) {
    // TODO: check we're in the "running" state
    // TODO
  }


  fn get_info(&self) -> Info {
    let params = self.params.lock();
    let latency = params.opus_codec.get_latency();

    Info {
      name: "Meltwater".to_string(),
      vendor: "Sarah Ocean".to_string(),
      presets: 1,
      parameters: NUM_PARAMETERS as i32,

      inputs: 2,
      outputs: 2,
      midi_inputs: 0,
      midi_outputs: 0,

      unique_id: 1559401526,
      version: 1,
      category: Category::Effect,

      initial_delay: latency as i32,
      preset_chunks: false,
      f64_precision: false,
      silent_when_stopped: false, // TODO: Should this be true?
    }
  }

  fn can_do(&self, can_do: CanDo) -> Supported {
    // TODO
    Supported::Maybe
  }

  fn get_tail_size(&self) -> isize {
    let params = self.params.lock();
    params.opus_codec.get_latency() as isize
  }
}

plugin_main!(MeltwaterPlugin);
