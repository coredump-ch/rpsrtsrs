extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use piston::input::{ Button };
use piston::input::mouse::MouseButton;

pub struct Square {
    rotation: f64,   // Rotation for the square.
    position: [f64; 2],
    target: [f64; 2],
    selected: bool,
}

impl Square {
    fn new(position: [f64;2]) -> Square {
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
    square: Square,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLUE:  [f32; 4] = [0.0, 0.0, 1.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.square.rotation;
        let pos = self.square.position;
        let selected = self.square.selected;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);

            let transform = c.transform.trans(pos[0], pos[1])
                                       .rot_rad(rotation)
                                       .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            if selected {
                rectangle(RED, square, transform, gl);
            } else {
                rectangle(BLUE, square, transform, gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.square.rotation += 2.0 * args.dt;

        let diff = [self.square.target[0]-self.square.position[0], self.square.target[1]-self.square.position[1]];
        self.square.position[0] += diff[0]/2.0*args.dt;
        self.square.position[1] += diff[1]/2.0*args.dt;
    }
}

fn main() {
    let opengl = OpenGL::_3_2;

    // Create an Glutin window.
    let window = Window::new(
        WindowSettings::new(
            "spinning-square",
            [200, 200]
        )
        .exit_on_esc(true)
    );

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        square: Square::new([50.0, 50.0]),
    };

    let mut cursor = [0.0,0.0];

    for e in window.events() {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }
        if let Some(Button::Mouse(button)) = e.press_args() {
            match button {
                MouseButton::Left  => println!("Pressed left mouse button '{:?}'", button),
                MouseButton::Right => app.square.target = cursor,
                _ => println!("Pressed mouse button '{:?}'", button),
            }
        }
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
            println!("Mouse moved '{} {}'", x, y);
        });

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}

