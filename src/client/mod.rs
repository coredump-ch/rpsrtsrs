use std::f64::consts::PI;
use opengl_graphics::GlGraphics;
use piston::input::{RenderArgs, UpdateArgs};

use shapes::Unit;

pub struct App {
    pub gl: GlGraphics, // OpenGL drawing backend.
    pub units: Vec<Unit>,
}

impl App {
    pub fn select(&mut self, position: [f64;2]) {
        for u in &mut self.units {
            u.selected = u.is_hit(position);
        };
    }

    pub fn render(&mut self, args: &RenderArgs) {
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

    pub fn update(&mut self, args: &UpdateArgs) {
        for s in &mut self.units {
            let diff = [s.target[0]-s.position[0], s.target[1]-s.position[1]];
            s.position[0] += diff[0]/2.0*args.dt;
            s.position[1] += diff[1]/2.0*args.dt;
        }
    }

    pub fn move_selected(&mut self, position: [f64;2]) {
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
