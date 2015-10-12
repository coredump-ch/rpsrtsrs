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

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    units: Vec<Unit>,
}

impl App {
    fn select(&mut self, position: &[f64;2]) {
        for s in &mut self.units {
            s.selected = position[0]< s.position[0]+25.0 &&
                position[0]> s.position[0]-25.0 &&
                position[1]< s.position[1]+25.0 &&
                position[1]> s.position[1]-25.0;
        };
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::{polygon, clear};
        use graphics::Transformed;
        use graphics::types::Polygon;

        const BLACK:  [f32; 4] = [0.0, 0.0,  0.0,  1.0];
        const YELLOW: [f32; 4] = [1.0, 1.0,  0.22, 1.0];
        const ORANGE: [f32; 4] = [1.0, 0.61, 0.22, 1.0];

        const FRONT_THICKNESS: f64 = 5.0;

        let units = &self.units;

        self.gl.draw(args.viewport(), |c, gl| {

            // Clear the screen.
            clear(BLACK, gl);

            for s in units.iter() {

                // Create a triangle polygon. The initial orientation is facing east.
                let triangle: Polygon = &s.get_shape();

                // Create a border on the front of the polygon. This is a trapezoid.
                // Because the angle of the trapezoid side is 22.5°, we know that `dx` is always `2 * dy`.
                let front: Polygon = &[
                    [s.size, s.size],                                           // Top right
                    [s.size, 0.0],                                                 // Bottom right
                    [s.size - FRONT_THICKNESS, FRONT_THICKNESS / 2.0],             // Bottom left
                    [s.size - FRONT_THICKNESS, s.size - FRONT_THICKNESS / 2.0], // Top left
                ];

                // Rotate the front to match the unit
                let transform_front = c.transform.trans(s.position[0], s.position[1])
                    .rot_rad(s.rotation)
                    .trans(-25.0, -25.0);

                // We don't need to apply any transformation to the units
                let transform_triangle = c.transform;

                // Draw the unit ORANGE if selected
                if s.selected {
                    polygon(ORANGE, triangle, transform_triangle, gl);
                    polygon(YELLOW, front, transform_front, gl);
                } else {
                    polygon(YELLOW, triangle, transform_triangle, gl);
                    polygon(ORANGE, front, transform_front, gl);
                }

            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        for s in &mut self.units {
            let diff = [s.target[0]-s.position[0], s.target[1]-s.position[1]];
            s.position[0] += diff[0]/2.0*args.dt;
            s.position[1] += diff[1]/2.0*args.dt;
        }
    }

    fn move_selected(&mut self, position: [f64;2]) {
        for s in &mut self.units {
            if s.selected {
                s.target = position;
                let dx = position[0] - s.position[0];
                let dy = position[1] - s.position[1];
                if dx.is_sign_negative() {
                    s.rotation = (dy / dx).atan() + PI;
                } else {
                    s.rotation = (dy / dx).atan();
                }
                println!("dx: {}, dy: {}, new rotation: {}", dx, dy, s.rotation);
            }
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let window : Window = WindowSettings::new(
        "rpsrtsrs",
        [640, 480]
    ).exit_on_esc(true).samples(8).into();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        units: vec![],
    };
    for _ in 0..10 {
        // Create new unit in random location
        let x = rand::random::<f64>() * 600.0 + 40.0;
        let y = rand::random::<f64>() * 440.0 + 40.0;
        let r = (rand::random::<f64>() - 0.5) * PI;
        let unit = Unit::new([x,y], r);

        // Register unit
        app.units.push(unit);
    }

    let mut cursor = [0.0,0.0];

    for e in window.events() {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }
        if let Some(Button::Mouse(button)) = e.press_args() {
            match button {
                MouseButton::Left  => app.select(&cursor),
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
