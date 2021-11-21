// Meltwater: Opus codec wrapper
// Copyright 2021, Sarah Ocean and the Meltwater project contributors.
// SPDX-License-Identifier: Apache-2.0

use audiopus::{Application, Bitrate, Channels, SampleRate};
use audiopus::coder::{Encoder, Decoder};

use crate::util::{interleave, deinterleave};

// Note on frame lengths: Opus only allows a handful of frame lengths (2.5ms,
// 5ms, 10ms, 20ms, 40ms, 60ms). Since we want to keep the latency as low as
// possible, we use 2.5ms frames (120 samples at Opus's internal rate of 48kHz),
// along with the special "low-delay" mode.
const FRAME_SIZE: usize = 120;

// TODO: Dynamically size buffers based on the host DAW's block size
const MAX_INPUT_BLOCK_SIZE: usize = 256;

// How much space to allocate for the intermediate packet buffer. The maximum
// bitrate we allow is 160kbps, which translates to an average of 400
// bits/packet == 50 bytes/packet.
//
// However, we want to allow for an occasional oversized packet, so we size the
// buffer significantly larger than the average
const MAX_PACKET_SIZE: usize = 128;

pub struct OpusCodec {
  // Buffer sizes are in samples, and each sample consists of two `f32` values
  input_buffer_size: usize,
  output_buffer_size: usize,

  // TODO: Use ring buffers for the input and output, to avoid having to move
  // data within the buffers
  // TODO also: Allow adjusting the size of these buffers based on the host
  // DAW's processing block size

  left_input: Vec<f32>,
  right_input: Vec<f32>,

  packet_samples: Vec<f32>,
  packet_data: Vec<u8>,

  left_output: Vec<f32>,
  right_output: Vec<f32>,

  input_samples_available: usize,
  output_samples_available: usize,

  encoder: Encoder,
  decoder: Decoder,
}

impl OpusCodec {
  pub fn new() -> Self {
    let input_buffer_size: usize = FRAME_SIZE + MAX_INPUT_BLOCK_SIZE;
    let packet_buffer_size: usize = MAX_PACKET_SIZE;
    let output_buffer_size: usize = FRAME_SIZE + MAX_INPUT_BLOCK_SIZE;

    let encoder = Encoder::new(
      SampleRate::Hz48000,
      Channels::Stereo,
      Application::LowDelay,
    ).unwrap();

    let decoder = Decoder::new(
      SampleRate::Hz48000,
      Channels::Stereo,
    ).unwrap();

    Self {
      input_buffer_size: input_buffer_size,
      output_buffer_size: output_buffer_size,

      left_input: vec![0f32; input_buffer_size],
      right_input: vec![0f32; input_buffer_size],
    
      // Opus takes input as interleaved stereo, so it needs 2 `f32`s per sample
      packet_samples: vec![0f32; FRAME_SIZE * 2],
      packet_data: vec![0u8; packet_buffer_size],
    
      left_output: vec![0f32; output_buffer_size],
      right_output: vec![0f32; output_buffer_size],

      input_samples_available: 0,
      // Initialize the output with one frame's worth of zeros, so that
      // we can use a simple "write N samples, process, read N samples" model
      // even if (for example) the first input block is < 1 frame in size
      output_samples_available: FRAME_SIZE,

      encoder: encoder,
      decoder: decoder,
    }
  }


  pub fn process_samples(&mut self, left_in: &[f32], right_in: &[f32],
                         left_out: &mut [f32], right_out: &mut [f32]) {
    self.load_input(left_in, right_in);
    self.process_frames();
    self.store_output(left_out, right_out);
  }

  fn load_input(&mut self, left: &[f32], right: &[f32]) {
    let num_samples = left.len();
    let input_buffer_start = self.input_samples_available;

    assert!(right.len() == num_samples);
    assert!(num_samples <= MAX_INPUT_BLOCK_SIZE);
    assert!(input_buffer_start + num_samples <= self.input_buffer_size);

    self.left_input[input_buffer_start .. input_buffer_start + num_samples]
        .copy_from_slice(left);
    self.right_input[input_buffer_start .. input_buffer_start + num_samples]
        .copy_from_slice(right);

    self.input_samples_available += num_samples;
  }

  fn process_frames(&mut self) {
    // Important: We might get multiple frames of data per call,
    // so loop until all available frames are processed
    while self.input_samples_available >= FRAME_SIZE {
      // Prepare input
      interleave(
        &self.left_input[0 .. FRAME_SIZE],
        &self.right_input[0 .. FRAME_SIZE],
        &mut self.packet_samples
      );

      self.left_input.copy_within(
        FRAME_SIZE .. self.input_samples_available,
        0,
      );
      self.right_input.copy_within(
        FRAME_SIZE .. self.input_samples_available,
        0,
      );
      self.input_samples_available -= FRAME_SIZE;

      // Encode then immediately decode
      let packet_size = self.encoder.encode_float(
        &self.packet_samples,
        &mut self.packet_data,
      ).unwrap();

      let num_decoded_samples = self.decoder.decode_float(
        Some(&self.packet_data[0..packet_size]),
        &mut self.packet_samples,
        false, // Do not apply error concealment
      ).unwrap();

      assert!(num_decoded_samples == FRAME_SIZE);

      // Transfer to output buffers
      let output_buffer_start = self.output_samples_available;
      assert!(output_buffer_start + FRAME_SIZE <= self.output_buffer_size);

      deinterleave(
        &self.packet_samples,
        &mut self.left_output[output_buffer_start .. output_buffer_start + FRAME_SIZE],
        &mut self.right_output[output_buffer_start .. output_buffer_start + FRAME_SIZE]
      );
      self.output_samples_available += FRAME_SIZE;
    }
  }

  fn store_output(&mut self, left: &mut [f32], right: &mut [f32]) {
    let num_samples = left.len();
    assert!(right.len() == num_samples);
    assert!(num_samples <= MAX_INPUT_BLOCK_SIZE);
    assert!(self.output_samples_available >= num_samples);

    left.copy_from_slice(&self.left_output[0 .. num_samples]);
    right.copy_from_slice(&self.right_output[0 .. num_samples]);

    self.left_output.copy_within(
      num_samples .. self.output_samples_available,
      0
    );
    self.right_output.copy_within(
      num_samples .. self.output_samples_available,
      0
    );

    self.output_samples_available -= num_samples;
  }


  pub fn get_latency(&self) -> u32 {
    let codec_latency = self.encoder.lookahead().unwrap();
    return (FRAME_SIZE as u32) + codec_latency;
  }

  pub fn set_bitrate(&mut self, bitrate_kbps: f32) {
    let bitrate = f32::round(bitrate_kbps * 1000.0) as i32;
    self.encoder.set_bitrate(Bitrate::BitsPerSecond(bitrate)).unwrap();
  }
}
