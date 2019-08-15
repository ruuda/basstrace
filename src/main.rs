// Basstrace -- Visualize room acoustics
// Copyright 2019 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

use std::env;
use std::sync::Arc;
use std::thread;

use gdk_pixbuf as gdk;
use gio::prelude::*;
use glib;
use gtk::prelude::*;

mod complex;
mod renderer;
mod scene;
mod vec2;
mod vec3;

use renderer::Renderer;
use scene::Scene;
use vec3::Vec3;

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

fn build_ui(application: &gtk::Application, renderer: &Arc<Renderer>) {
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
    let r_ref = renderer.clone();
    scale.connect_value_changed(move |scale_ref| {
        // Frequency = 10^slider_value.
        let log10_frequency = scale_ref.get_value() as f32;
        let frequency_hz = 2.0 * 10_f32.powf(log10_frequency);

        r_ref.set_frequency(frequency_hz);
    });

    let expand = true;
    let fill = false;
    let padding = 0;
    vbox.pack_start(&scale, expand, fill, padding);

    // Update the image every 5 seconds.
    let r_ref = renderer.clone();
    glib::source::timeout_add_seconds_local(5, move || {
        if let Some(mut pixbuf) = image.get_pixbuf() {
            r_ref.paint(&mut pixbuf);
            image.set_from_pixbuf(Some(&pixbuf));
        }
        glib::source::Continue(true)
    });

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(
        Some("nl.ruuda.basstrace"),
        Default::default(),
    ).unwrap();

    let renderer = Arc::new(Renderer::new());

    for _ in 0..4 {
        let r_ref = renderer.clone();
        thread::spawn(move || {
            r_ref.run_render_loop();
        });
    }

    application.connect_activate(move |app| {
        build_ui(app, &renderer);
    });

    let args: Vec<_> = env::args().collect();
    application.run(&args);
}
