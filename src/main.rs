// Basstrace -- Visualize room acoustics
// Copyright 2019 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

extern crate gio;
extern crate gtk;
extern crate gdk_pixbuf;

use gio::prelude::*;
use gtk::prelude::*;
use gdk_pixbuf as gdk;

use std::env;

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

fn paint(pixbuf: &mut gdk::Pixbuf) {

    for y in 0..720 {
        for x in 0..1280 {
            let r = if y & 1 == 1 { 255 } else { 0 };
            let g = if x & 1 == 1 { 255 } else { 0 };
            let b = 0;
            let a = 255;
            pixbuf.put_pixel(x, y, r, g, b, a);
        }
    }
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    let canvas = build_canvas();
    let image = gtk::Image::new_from_pixbuf(canvas.as_ref());

    window.set_title("Basstrace");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1280, 720);

    if let Some(mut pixbuf) = image.get_pixbuf() {
        paint(&mut pixbuf);
    }

    window.add(&image);

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
