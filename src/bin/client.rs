extern crate env_logger;
extern crate piston;
#[macro_use]
extern crate serde_derive;
extern crate docopt;
#[cfg(feature = "include_glfw")]
extern crate glfw_window;
#[cfg(feature = "include_glutin")]
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate rand;
extern crate rpsrtsrs;
#[cfg(feature = "include_sdl2")]
extern crate sdl2_window;
extern crate texture;

use std::path::Path;

#[cfg(feature = "include_glfw")]
use glfw_window::GlfwWindow as Window;
#[cfg(feature = "include_glutin")]
use glutin_window::GlutinWindow as Window;
use opengl_graphics::GlyphCache;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
#[cfg(feature = "include_sdl2")]
use sdl2_window::Sdl2Window as Window;
use texture::TextureSettings;

use rpsrtsrs::client::*;

static USAGE: &'static str = "
Usage: client [-p PORT] [-i IP]

Options:
    -p PORT  The port to listen on [default: 8080].
    -i IP    The ipv4 address to listen on [default: 127.0.0.1].
    -r ID    Reconnect with the given ID
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_p: u16,
    flag_i: String,
}

fn main() {
    env_logger::init();
    let opengl = OpenGL::V3_2;

    let args: Args = docopt::Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("rpsrtsrs", [640, 480])
        .exit_on_esc(true)
        .samples(8)
        .build()
        .unwrap();

    let font_path = Path::new("assets/DejaVuSans.ttf");
    let texture_settings = TextureSettings::new();
    let ref mut cache = GlyphCache::new(font_path, (), texture_settings).unwrap();

    // Create a new game and run it.
    let mut app = App::new(GlGraphics::new(opengl), args.flag_i, args.flag_p);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r, cache);
        }

        if let Some(button) = e.press_args() {
            if app.on_button_press(&button) {
                break;
            }
        }

        if let Some(args) = e.mouse_cursor_args() {
            app.on_mouse_move(args.into());
        }

        if let Some(args) = e.mouse_scroll_args() {
            app.on_mouse_scroll(args.into());
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}
