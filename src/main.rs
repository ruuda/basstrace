// Basstrace -- Visualize room acoustics
// Copyright 2019 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::env;

use gio::prelude::*;
use gtk::prelude::*;
use gdk_pixbuf as gdk;

mod complex;
mod scene;
mod vec2;

use scene::Scene;
use vec2::Vec2;

fn build_canvas() -> Option<gdk::Pixbuf> {
    let has_alpha = false;
    let bits_per_sample = 8;
    let width = 1280;
    let height = 720;
    gdk::Pixbuf::new(
        gdk::Colorspace::Rgb,
        has_alpha,
        bits_per_sample,
        width,
        height,
    )
}

fn paint(scene: &Scene, frequency: f32, pixbuf: &mut gdk::Pixbuf) {
    for y in 0..720 {
        let ym = (y - 360) as f32 * 0.01;

        for x in 0..1280 {
            let xm = (x - 640) as f32 * 0.01;

            let position = Vec2::new(xm, ym);
            let magnitude = scene.sample_at(frequency, position).norm().log10();
            let rf = (0.5 + magnitude * 0.2).max(0.0).min(1.0);

            let r = (rf * 255.0) as u8;
            let g = r;
            let b = r;
            let a = 255;
            pixbuf.put_pixel(x, y, r, g, b, a);
        }
    }
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Basstrace");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1280, 720);

    let vbox = gtk::Box::new(
        gtk::Orientation::Vertical,
        10,
    );
    window.add(&vbox);

    let canvas = build_canvas();
    let image = gtk::Image::new_from_pixbuf(canvas.as_ref());

    let expand = false;
    let fill = false;
    let padding = 0;
    vbox.pack_start(&image, expand, fill, padding);

    let min = 1.0;
    let max = 4.0;
    let step = 0.05;
    let scale = gtk::Scale::new_with_range(
        gtk::Orientation::Horizontal,
        min, max, step,
    );
    scale.connect_value_changed(move |scale_ref| {
        // Frequency = 10^slider_value.
        let log10_frequency = scale_ref.get_value() as f32;
        let frequency = 2.0 * 10_f32.powf(log10_frequency);

        if let Some(mut pixbuf) = image.get_pixbuf() {
            let scene = Scene::new_example();
            paint(&scene, frequency, &mut pixbuf);
            image.set_from_pixbuf(Some(&pixbuf));
        }
    });

    let expand = true;
    let fill = false;
    let padding = 0;
    vbox.pack_start(&scale, expand, fill, padding);

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(
        Some("nl.ruuda.basstrace"),
        Default::default(),
    ).unwrap();

    application.connect_activate(|app| {
        build_ui(app);
    });

    let args: Vec<_> = env::args().collect();
    application.run(&args);
}
