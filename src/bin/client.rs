extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
extern crate rand;
#[cfg(feature = "include_sdl2")] extern crate sdl2_window;
#[cfg(feature = "include_glfw")] extern crate glfw_window;
#[cfg(feature = "include_glutin")] extern crate glutin_window;

use piston::window::WindowSettings;
use opengl_graphics::{ GlGraphics, OpenGL };
use piston::input::*;
use piston::event_loop::*;
#[cfg(feature = "include_sdl2")] use sdl2_window::Sdl2Window as Window;
#[cfg(feature = "include_glfw")] use glfw_window::GlfwWindow as Window;
#[cfg(feature = "include_glutin")] use glutin_window::GlutinWindow as Window;

pub struct Square {
    rotation: f64,   // Rotation for the square.
    position: [f64; 2],
    target: [f64; 2],
    selected: bool,
}

impl Square {
    fn new(position: [f64;2]) -> Square {
        println!("Create square at {:?}", position);
        Square {
            rotation: 0.0,
            position: position,
            target: position,
            selected: false
        }
    }
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    squares: Vec<Square>,
}

impl App {
    fn select(&mut self, position: &[f64;2]) {
        for s in &mut self.squares {
            s.selected = position[0]< s.position[0]+25.0 &&
                position[0]> s.position[0]-25.0 &&
                position[1]< s.position[1]+25.0 &&
                position[1]> s.position[1]-25.0;
        };
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0,  0.0, 1.0];
        const YELLOW:[f32; 4] = [1.0, 1.0,  0.22, 1.0];
        const ORANGE:[f32; 4] = [1.0, 0.61, 0.22, 1.0];

        let squares = &self.squares;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);
            for s in squares.iter() {
                let square = rectangle::square(0.0, 0.0, 50.0);
                let transform = c.transform.trans(s.position[0], s.position[1])
                    .rot_rad(s.rotation)
                    .trans(-25.0, -25.0);

                // Draw the box RED if selected
                if s.selected {
                    rectangle(ORANGE, square, transform, gl);
                } else {
                    rectangle(YELLOW, square, transform, gl);
                }
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        for s in &mut self.squares {
            let diff = [s.target[0]-s.position[0], s.target[1]-s.position[1]];
            s.position[0] += diff[0]/2.0*args.dt;
            s.position[1] += diff[1]/2.0*args.dt;
        }
    }

    fn move_selected(&mut self, position: [f64;2]) {
        for s in &mut self.squares {
            if s.selected {
                s.target = position;
                let d0 = position[0]-s.position[0];
                let d1 = position[1]-s.position[1];
                s.rotation = (d1/d0).atan();
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
    ).exit_on_esc(true).into();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        squares: vec![],
    };
    for _ in 0..10 {
        let x = rand::random::<f64>() * 600.0 + 40.0;
        let y = rand::random::<f64>() * 440.0 + 40.0;
        app.squares.push(Square::new([x,y]));
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

