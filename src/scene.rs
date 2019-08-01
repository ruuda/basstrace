// Basstrace -- Visualize room acoustics
// Copyright 2019 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::f32::consts::PI;

use crate::complex::Complex;
use crate::vec2::Vec2;

/// The speed of sound in m/s in air at 25 degrees Celsius and 1 atm.
/// TODO: Parametrize temperature and pressure.
const SPEED_OF_SOUND: f32 = 346.3;

pub struct Source {
    position: Vec2,
}

impl Source {
    /// Sample the field produced by the source at the given position.
    ///
    /// * `frequency` specifies the source frequency in Hz.
    /// * `position` specifies the position measured in meters from the origin.
    pub fn sample_at(&self, frequency: f32, position: Vec2) -> Complex {
        let distance_squared = (position - self.position).norm_squared();
        let n_waves = frequency * distance_squared.sqrt() / SPEED_OF_SOUND;
        // NOTE: Should be 1/d in 2d, but I want to do 3d eventually, where it
        // should be 1/d^2.
        let amplitude = distance_squared.recip();
        Complex::exp_i(2.0 * PI * n_waves) * amplitude
    }
}

pub struct Scene {
    sources: Vec<Source>,
}

impl Scene {
    /// Return an example scene with hard-coded data.
    pub fn new_example() -> Scene {
        Scene {
            sources: vec![
                Source { position: Vec2::new(-1.65, -1.0) },
                Source { position: Vec2::new( 1.65, -1.0) },
            ],
        }
    }

    /// See `Source::sample_at()`.
    pub fn sample_at(&self, frequency: f32, position: Vec2) -> Complex {
        let mut z = Complex::zero();
        for s in &self.sources {
            z = z + s.sample_at(frequency, position);

            // Create a reflection at y = -2.0.
            let y_wall = -2.0;
            if position.y < y_wall {
                z = Complex::zero();
                continue;
            } else {
                let reflected_pos = Vec2::new(position.x, y_wall - (position.y - y_wall));
                z = z + s.sample_at(frequency, reflected_pos);
            }

            let y_wall = 3.0;
            if position.y > y_wall {
                z = Complex::zero();
                continue;
            } else {
                let reflected_pos = Vec2::new(position.x, y_wall - (position.y - y_wall));
                z = z + s.sample_at(frequency, reflected_pos);
            }

            let x_wall = -2.5;
            if position.x < x_wall {
                z = Complex::zero();
                continue;
            } else {
                let reflected_pos = Vec2::new(x_wall - (position.x - x_wall), position.y);
                z = z + s.sample_at(frequency, reflected_pos);
            }

            let x_wall = 6.0;
            if position.x > x_wall {
                z = Complex::zero();
                continue;
            } else {
                let reflected_pos = Vec2::new(x_wall - (position.x - x_wall), position.y);
                z = z + s.sample_at(frequency, reflected_pos);
            }
        }
        z
    }
}
