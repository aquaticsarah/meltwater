// Meltwater: Utility functions
// Copyright 2021, Sarah Ocean and the Meltwater project contributors.
// SPDX-License-Identifier: Apache-2.0

// Convert separate left and right channels <-> interleaved stereo
pub fn interleave(left: &[f32], right: &[f32], output: &mut [f32]) {
  let num_samples = left.len();
  assert!(right.len() == num_samples);
  assert!(output.len() == 2 * num_samples);

  for i in 0 .. num_samples {
    output[2*i + 0] = left[i];
    output[2*i + 1] = right[i];
  }
}

pub fn deinterleave(input: &[f32], left: &mut [f32], right: &mut [f32]) {
  let num_samples = left.len();
  assert!(right.len() == num_samples);
  assert!(input.len() == 2 * num_samples);

  for i in 0 .. num_samples {
    left[i] = input[2*i + 0];
    right[i] = input[2*i + 1];
  }
}
