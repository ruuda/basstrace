// Basstrace -- Visualize room acoustics
// Copyright 2019 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

/// The splitmix64 pseudorandom number generator.
///
/// Translated from http://prng.di.unimi.it/splitmix64.c, which is licensed CC0.
#[inline]
pub fn splitmix64_next(state: &mut u64) -> u64 {
    *state += 0x9e3779b97f4a7c15;
    let mut z = *state;
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9;
    z = (z ^ (z >> 27)) * 0x94d049bb133111eb;
    z ^ (z >> 31)
}

/// The xoshiro256++ pseudorandom number generator.
///
/// Translated from http://prng.di.unimi.it/xoshiro256plusplus.c, which is licensed CC0.
#[inline]
pub fn xoshiro256pp_next(state: &mut [u64; 4]) -> u64 {
    let result = (state[0] + state[3]).rotate_left(23) + state[0];

    let t = state[1] << 17;

    state[2] ^= state[0];
    state[3] ^= state[1];
    state[1] ^= state[2];
    state[0] ^= state[3];

    state[2] ^= t;

    state[3] = state[3].rotate_left(45);

    result
}

/// An pseudorandom number generator powered by xoshiro256++.
pub struct Rng {
  state: [u64; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        let mut sp64_state = seed;
        Rng {
            // Seed the generator with splitmix64,
            // as recommended by the authors.
            state: [
                splitmix64_next(&mut sp64_state),
                splitmix64_next(&mut sp64_state),
                splitmix64_next(&mut sp64_state),
                splitmix64_next(&mut sp64_state),
            ],
        }
    }

    #[inline]
    pub fn next(&mut self) -> u64 {
        xoshiro256pp_next(&mut self.state)
    }
}
