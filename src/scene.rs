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
    position: Vec3,
    direction: Vec3,
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

        let reflectivity = -0.8;

        let mut z = Complex::zero();

        for s in &self.sources {
            // Order 0: direct.
            let p0 = position;
            z = z + s.sample_at(frequency, p0);

            for i in 0..self.faces.len() {
                // Order 1: after a single reflection.
                let p1 = self.faces[i].reflect(p0);
                z = z + s.sample_at(frequency, p1) * reflectivity;

                for j in 0..self.faces.len() {
                    if i == j { continue }
                    // Order 2: after two reflections.
                    let p2 = self.faces[j].reflect(p1);
                    z = z + s.sample_at(frequency, p2) * (reflectivity * reflectivity);

                    for k in 0..self.faces.len() {
                        if j == k { continue }
                        // Order 3: after three reflections.
                        let p3 = self.faces[k].reflect(p2);
                        z = z + s.sample_at(frequency, p3) * (reflectivity * reflectivity * reflectivity);

                        /*for m in 0..self.faces.len() {
                            if k == m { continue }
                            // Order 4: after four reflections.
                            let p4 = self.faces[m].reflect(p3);
                            z = z + s.sample_at(frequency, p4) * (reflectivity * reflectivity * reflectivity * reflectivity);
                        }*/
                    }
                }
            }
        }

        z
    }
}
