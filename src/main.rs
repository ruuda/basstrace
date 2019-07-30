// Basstrace -- Visualize room acoustics
// Copyright 2019 Ruud van Asseldonk

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3. A copy
// of the License is available in the root of the repository.

extern crate gio;
extern crate gtk;

use gio::prelude::*;
use gtk::prelude::*;

use std::env;

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Basstrace");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1280, 720);

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
