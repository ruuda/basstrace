// Basstrace -- Visualize room acoustics
// Copyright 2019 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::ops;

use crate::vec2::Vec2;

/// Represents a complex number.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Complex(pub Vec2);

impl Complex {
    pub fn new(real: f32, imag: f32) -> Complex {
        Complex(Vec2::new(real, imag))
    }

    pub fn zero() -> Complex {
        Complex(Vec2::zero())
    }

    /// Return `exp(i * t)`.
    pub fn exp_i(t: f32) -> Complex {
        Complex(Vec2::new(t.cos(), t.sin()))
    }

    pub fn real(&self) -> f32 {
        self.0.x
    }

    pub fn imag(&self) -> f32 {
        self.0.y
    }

    pub fn norm(&self) -> f32 {
        self.0.norm()
    }
}

impl ops::Add for Complex {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Complex(self.0 + other.0)
    }
}

impl ops::Sub for Complex {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Complex(self.0 - other.0)
    }
}

impl ops::Mul<f32> for Complex {
    type Output = Complex;

    fn mul(self, other: f32) -> Self {
        Complex(self.0 * other)
    }
}

impl ops::Mul<Self> for Complex {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Complex::new(
            self.real() * other.real() - self.imag() * other.imag(),
            self.real() * other.imag() + self.imag() * other.real(),
        )
    }
}
