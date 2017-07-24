extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
extern crate rand;
extern crate rpsrtsrs;
#[cfg(feature = "include_sdl2")] extern crate sdl2_window;
#[cfg(feature = "include_glfw")] extern crate glfw_window;
#[cfg(feature = "include_glutin")] extern crate glutin_window;


use std::f64::consts::PI;
use piston::window::WindowSettings;
use opengl_graphics::{ GlGraphics, OpenGL };
use piston::input::*;
use piston::event_loop::*;
#[cfg(feature = "include_sdl2")] use sdl2_window::Sdl2Window as Window;
#[cfg(feature = "include_glfw")] use glfw_window::GlfwWindow as Window;
#[cfg(feature = "include_glutin")] use glutin_window::GlutinWindow as Window;
use rpsrtsrs::shapes::Unit;
use rpsrtsrs::client::*;


fn main() {
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window : Window = WindowSettings::new(
        "rpsrtsrs",
        [640, 480]
    ).exit_on_esc(true).samples(8).build().unwrap();

    // Create a new game and run it.
    let mut app = App::new(GlGraphics::new(opengl));
    let world_state = app.world_state.clone();
    let commands = app.commands.clone();
    let mut network_client = NetworkClient::new(("127.0.0.1", 8080), world_state.clone(), commands);

    network_client.connect();
    network_client.update();


    let mut cursor = [0.0,0.0];

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {

        let world_lock = world_state.lock().unwrap();
        //app.units = vec![];
        if let Some(player) = world_lock.game.players.get(0) {
            for unit in player.units.iter() {
                app.units.get_mut(unit.id.0 as usize)
                    .map(|app_unit| app_unit.position = unit.position)
                    .or_else(|| {
                        app.units.push(Unit::new(unit.position.clone(), 0.0f64));
                        None
                    }
                    );
            }
        }

        if let Some(r) = e.render_args() {
            app.render(&r);
        }
        if let Some(Button::Mouse(button)) = e.press_args() {
            match button {
                MouseButton::Left  => app.select(cursor),
                MouseButton::Right => app.move_selected(cursor),
                _ => println!("Pressed mouse button '{:?}'", button),
            }
        }
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
        });

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}
