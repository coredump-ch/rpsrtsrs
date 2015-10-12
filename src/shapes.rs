//! Shapes in the game, like units (triangles) and buildings (squares).
extern crate graphics;
use self::graphics::types::Triangle;
use self::graphics::math::Matrix2d;
use self::graphics::math;


pub struct Unit {
    pub rotation: f64,
    pub position: [f64; 2],
    pub target: [f64; 2],
    pub selected: bool,
    pub size: f64,
}


impl Unit {

    pub fn new(position: [f64;2], rotation: f64) -> Unit {
        println!("Create unit at {:?} with rotation {}", position, rotation);
        Unit {
            rotation: rotation,
            position: position,
            target: position,
            selected: false,
            size: 50.0,
        }
    }

    pub fn get_shape(&self) -> Triangle {
        // Base shape
        let mut triangle: Triangle = [
            [0.0, self.size / 2.0], // Left
            [self.size, self.size], // Top right
            [self.size, 0.0],       // Bottom right
        ];

        // Transformations
        for point in triangle.iter_mut() {
            // Translate center to zero point
            *point = math::add(*point, [-self.size / 2.0, -self.size / 2.0]);

            // Rotate
            let rotation_matrix = math::rotate_radians(self.rotation);
            *point = math::transform_vec(rotation_matrix, *point);

            // Translate to effective position
            *point = math::add(*point, self.position);
        }

        triangle
    }

}
