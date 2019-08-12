// Basstrace -- Visualize room acoustics
// Copyright 2019 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::f32::consts::PI;

use crate::complex::Complex;
use crate::vec3::Vec3;

/// The speed of sound in m/s in air at 25 degrees Celsius and 1 atm.
/// TODO: Parametrize temperature and pressure.
const SPEED_OF_SOUND: f32 = 346.3;

pub struct Source {
    position: Vec3,
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

impl Source {
    /// Sample the field produced by the source at the given position.
    ///
    /// * `frequency` specifies the source frequency in Hz.
    /// * `position` specifies the position measured in meters from the origin.
    pub fn sample_at(&self, frequency: f32, position: Vec3) -> Complex {
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
    faces: Vec<Face>,
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

        let ceil_off = Vec3::new(0.0, 0.0, 2.8);

        Scene {
            sources: vec![
                Source { position: Vec3::new(0.60, 0.30, 1.0) },
                Source { position: Vec3::new(2.20, 0.30, 1.0) },
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
    pub fn sample_at(&self, frequency: f32, position: Vec3) -> Complex {
        let mut z = Complex::zero();
        for s in &self.sources {
            z = z + s.sample_at(frequency, position);

            // Sample first order reflections.
            for face in &self.faces {
                if face.is_facing(position) {
                    let reflected_pos = face.reflect(position);
                    z = z + s.sample_at(frequency, reflected_pos);
                } else {
                    z = Complex::zero();
                    break
                }
            }
        }
        z
    }
}
