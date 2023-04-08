mod file;

use gtk::prelude::*;
use gtk::{Window, WindowType};

fn main() {
    // Initialize GTK.
    gtk::init().unwrap();

    // Create a new window.
    let window = Window::new(WindowType::Toplevel);

    // Set the title of the window.
    window.set_title("My Rust Window");

    // Set the default size of the window.
    window.set_default_size(1600, 1400);

    // Connect the "destroy" signal to quit the program when the window is closed.
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    // Show the window.
    window.show_all();

    // Start the GTK main loop.
    gtk::main();
}
