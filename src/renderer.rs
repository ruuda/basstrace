// Basstrace -- Visualize room acoustics
// Copyright 2019 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::sync::Mutex;
use std::iter;

use gdk_pixbuf as gdk;

use crate::complex::Complex;
use crate::scene::Scene;
use crate::vec3::Vec3;

#[derive(Copy, Clone, PartialEq)]
struct RenderParams {
    frequency_hz: f32,
}

pub struct Renderer {
    scene: Scene,
    width: u32,
    height: u32,
    params: Mutex<RenderParams>,
    buffer: Mutex<Vec<Complex>>,
}

impl Renderer {
    pub fn new() -> Renderer {
        let params = RenderParams {
            frequency_hz: 440.0,
        };

        let width = 1280;
        let height = 720;

        let buffer: Vec<_> = iter::repeat(Complex::zero())
            .take(width * height)
            .collect();

        Renderer {
            scene: Scene::new_example(),
            width: width as u32,
            height: height as u32,
            params: Mutex::new(params),
            buffer: Mutex::new(buffer),
        }
    }

    #[inline]
    fn area(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub fn clear(&self) {
        let mut b = self.buffer.lock().unwrap();
        for z in b.iter_mut() {
            *z = Complex::zero();
        }
    }

    pub fn set_frequency(&self, f_hz: f32) {
        let mut p = self.params.lock().unwrap();
        p.frequency_hz = f_hz;
        self.clear();
    }

    /// Add `buffer` to the internal buffer, and zero `buffer` itself.
    ///
    /// In a sense, move the density out of `buffer` into `self.buffer`. Takes
    /// the render params to confirm that they are the same as the values that
    /// `buffer` was filled for; we would not want to merge a stale render.
    fn accumulate_move(&self, params: &RenderParams, buffer: &mut [Complex]) {
        assert_eq!(buffer.len(), self.area());

        // Only accumulate if the values we want to add were computed for the
        // same parameters.
        {
            let p = self.params.lock().unwrap();
            if *p != *params {
                return
            }
        }

        let mut b = self.buffer.lock().unwrap();
        assert_eq!(b.len(), buffer.len());

        for (dst, src) in b.iter_mut().zip(buffer.iter_mut()) {
            *dst = *dst + *src;
            *src = Complex::zero();
        }
    }

    pub fn run_render_loop(&self) {
        let mut buffer: Vec<_> = iter::repeat(Complex::zero())
            .take(self.area())
            .collect();

        loop {
            let params = self.params.lock().unwrap().clone();
            render_one(&self.scene, &params, &mut buffer[..], self.width, self.height);
            self.accumulate_move(&params, &mut buffer[..]);
        }
    }

    pub fn paint(&self, pixbuf: &mut gdk::Pixbuf) {
        let buffer = self.buffer.lock().unwrap();
        assert_eq!(buffer.len(), self.area());

        for y in 0..self.height {
            for x in 0..self.width {
                let i = y * self.width + x;

                let magnitude = buffer[i as usize].norm().log10();
                let rf = (0.5 + magnitude * 0.2).max(0.0).min(1.0);

                let r = (rf * 255.0) as u8;
                let g = r;
                let b = r;
                let a = 255;

                pixbuf.put_pixel(x as i32, y as i32, r, g, b, a);
            }
        }
    }
}

fn render_one(
    scene: &Scene,
    params: &RenderParams,
    buffer: &mut [Complex],
    width: u32,
    height: u32,
) {
    for y in 0..height {
        let ym = y as f32 * 0.008;

        for x in 0..width {
            let i = (y * width + x) as usize;

            let xm = x as f32 * 0.008;
            let position = Vec3::new(xm - 0.5, ym - 0.5, 1.0);
            buffer[i] = scene.sample_at(params.frequency_hz, position);
        }
    }
}
