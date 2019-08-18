// Basstrace -- Visualize room acoustics
// Copyright 2019 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::f32::consts::PI;

use crate::complex::Complex;
use crate::vec3::Vec3;
use crate::rand::Rng;

/// The speed of sound in m/s in air at 25 degrees Celsius and 1 atm.
/// TODO: Parametrize temperature and pressure.
const SPEED_OF_SOUND: f32 = 346.3;

/// A speaker, emitting sound in the given direction.
pub struct Source {
    pub position: Vec3,
    pub direction: Vec3,
}

impl Source {
    pub fn new(position: Vec3, aimed_at: Vec3) -> Source {
        Source {
            position: position,
            direction: (aimed_at - position).normalized(),
        }
    }

    /// Sample the field produced by the source at the given position.
    ///
    /// * `frequency` specifies the source frequency in Hz.
    /// * `position` specifies the position measured in meters from the origin.
    pub fn sample_at(&self, frequency: f32, position: Vec3) -> Complex {
        // The energy falls off with radius squared.
        let distance_squared = (position - self.position).norm_squared();
        let attenuation_distance = distance_squared.recip();

        // The phase is proportional to the distance.
        let distance = distance_squared.sqrt();
        let n_waves = frequency * distance / SPEED_OF_SOUND;

        // Furthermore, if we are behind the speaker, the phase is inverted, and
        // we assume that the speaker does not emit sound sideways. We model
        // this with another attenuation factor, proportional to the dot product
        // between the normalized direction to the target, and speaker output
        // direction.
        let dot = (position - self.position).dot(self.direction);
        let attenuation_phase = dot * distance.recip();

        Complex::exp_i(2.0 * PI * n_waves) * attenuation_distance * attenuation_phase
    }
}

/// An (infinite) band, the intersection of two half-planes.
///
/// * The band is a subset of the plane through `origin` perpendicular to `normal`.
/// * The band is bounded from one side by the line (in the plane) through
///   `origin`, perpendicular to both `normal` and `tangent`.
/// * The band is bounded from the other side by the line (in the plane) through
///   `origin + width * tangent` perpendicular to both `normal` and `tangent`.
pub struct Face {
    origin: Vec3,
    normal: Vec3,
    tangent: Vec3,
    width: f32,
}

impl Face {
    /// Construct a face through `p1` and `p2`.
    ///
    /// * The normal and tangent will be perpendicular to `forward`.
    /// * The tangent will point from `p1` to `p2`.
    pub fn new(p1: Vec3, p2: Vec3, forward: Vec3) -> Face {
        Face {
            origin: p1,
            normal: forward.cross(p2 - p1).normalized(),
            tangent: (p2 - p1).normalized(),
            width: (p2 - p1).norm(),
        }
    }

    /// Reflect the point p in the plane.
    pub fn reflect(&self, p: Vec3) -> Vec3 {
        let d = self.normal.dot(p - self.origin);
        p - self.normal * (d + d)
    }

    /// Return whether the point is on the front side of the face.
    ///
    /// The front side of the face is the side where the normal points.
    pub fn is_facing(&self, p: Vec3) -> bool {
        self.normal.dot(p - self.origin) > 0.0
    }
}

pub struct Scene {
    pub sources: Vec<Source>,
    pub faces: Vec<Face>,
}

impl Scene {
    /// Return an example scene with hard-coded data.
    pub fn new_example() -> Scene {
        let up = Vec3::new(0.0, 0.0, 1.0);
        let side = Vec3::new(0.0, 1.0, 0.0);

        let p0 = Vec3::new(0.00, 0.00, 0.0);
        let p1 = Vec3::new(8.32, 0.00, 0.0);
        let p2 = Vec3::new(8.32, 3.35, 0.0);
        let p3 = Vec3::new(0.00, 3.35, 0.0);

        let s1 = Vec3::new(0.60, 0.30, 1.0);
        let s2 = Vec3::new(2.20, 0.30, 1.0);
        let listener = Vec3::new(1.40, 3.0, 1.0);

        let ceil_off = Vec3::new(0.0, 0.0, 2.8);

        Scene {
            sources: vec![
                Source::new(s1, listener),
                Source::new(s2, listener),
            ],

            faces: vec![
                // Walls.
                Face::new(p0, p1, up),
                Face::new(p1, p2, up),
                Face::new(p2, p3, up),
                Face::new(p3, p0, up),

                // Floor and ceiling.
                Face::new(p0, p1, -side),
                Face::new(p0 + ceil_off, p1 + ceil_off, side),
            ],
        }
    }

    /// See `Source::sample_at()`.
    pub fn sample_at(&self, rng: &mut Rng, frequency: f32, position: Vec3) -> Complex {
        // Sample first order reflections.
        for face in &self.faces {
            if !face.is_facing(position) {
                return Complex::zero();
            }
        }

        let reflectivity = -0.95;

        // The incoming energy is the sum over all paths that start at the
        // source and end at the listener. We can partition the set of all paths
        // by the number of bounces, such that the sum is the sum over n from 0
        // to infinity, of the contribution from paths with n bounces. Below we
        // only evaluate the contributions up to some finite n, and we assume
        // that beyond that, the contributions are small enough to be negligible.
        // For a given n, we can enumerate the set of paths with n bounces: at
        // n=0 we have a direct path, at n=1 we can bounce via any of the faces,
        // at n=1 we can bounce via any of the faces first, and then through a
        // different face, etc. The number of paths blows up as num_faces^n, so
        // enumerating them quickly becomes infeasible; we need to sample. And
        // while we're sampling for a given n, we need to compute the
        // reflections for n-1 anyway, so we might as well sample n-1 at the
        // same time. The set of paths with n+1 bounces is num_faces-1 times as
        // large as the set of paths with n bounces, so for every path with n+1
        // bounces, if we take its prefix of n bounces into account too, then
        // the weight of the path with n+1 bounces should be num_faces-1 times
        // as large.
        let factor = reflectivity * (self.faces.len() - 1) as f32;

        let si = rng.index(&self.sources[..]);
        let source = &self.sources[si];

        let mut z = Complex::zero();
        let mut p = position;
        let mut amplitude = 1.0 / 4096.0;
        let mut fi = rng.index(&self.faces[..]);

        // We go for up to 56 bounces. With walls of 3m long, that amounts to
        // about 500ms.
        for bounce in 0..30 {
            // Directly, from source to listener.
            let m = source.sample_at(frequency, p);
            z = z + m * amplitude;

            // Pick a face to reflect from, which should not be the same face
            // that we reflected from last time.
            loop {
                let next_fi = rng.index(&self.faces[..]);
                if next_fi != fi {
                    fi = next_fi;
                    break;
                }
            }

            let face = &self.faces[fi];
            p = face.reflect(p);
            amplitude *= factor;
        }

        z
    }
}
