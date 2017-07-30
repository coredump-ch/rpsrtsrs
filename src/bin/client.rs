extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
extern crate texture;
extern crate rand;
extern crate rpsrtsrs;
#[cfg(feature = "include_sdl2")] extern crate sdl2_window;
#[cfg(feature = "include_glfw")] extern crate glfw_window;
#[cfg(feature = "include_glutin")] extern crate glutin_window;

use std::path::Path;

use piston::window::WindowSettings;
use opengl_graphics::{ GlGraphics, OpenGL };
use opengl_graphics::glyph_cache::GlyphCache;
use texture::TextureSettings;
use piston::input::*;
use piston::event_loop::*;
#[cfg(feature = "include_sdl2")] use sdl2_window::Sdl2Window as Window;
#[cfg(feature = "include_glfw")] use glfw_window::GlfwWindow as Window;
#[cfg(feature = "include_glutin")] use glutin_window::GlutinWindow as Window;

use rpsrtsrs::client::*;


fn main() {
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window : Window = WindowSettings::new(
        "rpsrtsrs",
        [640, 480]
    ).exit_on_esc(true).samples(8).build().unwrap();


    let font_path = Path::new("assets/DejaVuSans.ttf");
    let texture_settings = TextureSettings::new();
    let ref mut cache = GlyphCache::new(font_path, texture_settings).unwrap();

    // Create a new game and run it.
    let mut app = App::new(GlGraphics::new(opengl));

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {

        if let Some(r) = e.render_args() {
            app.render(&r, cache);
        }

        if let Some(button) = e.press_args() {
            app.on_button_press(&button);
        }

        if let Some(args) = e.mouse_cursor_args() {
            app.on_mouse_move(args);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}
