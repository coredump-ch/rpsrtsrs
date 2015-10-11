//! Shapes in the game, like units (triangles) and buildings (squares).
extern crate graphics;
use self::graphics::types::Triangle;
use self::graphics::math::{rotate_radians, transform_vec, Matrix2d};


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
        let triangle: Triangle = [
            [0.0, self.size / 2.0], // Left
            [self.size, self.size], // Top right
            [self.size, 0.0],       // Bottom right
        ];
        triangle
    }

}

/// Apply a transformation matrix to all points in the triangle.
fn apply_matrix(triangle: Triangle, m: Matrix2d) -> Triangle {
    [
        transform_vec(m, triangle[0]),
        transform_vec(m, triangle[1]),
        transform_vec(m, triangle[2]),
    ]
}


#[cfg(test)]
mod test {
    use std::f64::consts::FRAC_PI_2;
    use super::graphics::types::Triangle;
    use super::graphics::math::rotate_radians;
    use super::apply_matrix;

    #[test]
    fn test_apply_matrix() {
        //! Verify that the `apply_matrix` function works properly, by applying a
        //! 90° CCW rotation matrix.
        //!
        //! The initial triangle should look like this:
        //!
        //!       2
        //!        |\
        //!        |_\
        //!       1   3
        //!
        //! After the translation, it should be rotated to the left.
        //!
        //!        3
        //!       /|
        //!      /_|
        //!     2   1

        // Initial triangle.
        let t1: Triangle = [
            [0.0, 0.0],
            [0.0, 5.0],
            [5.0, 0.0],
        ];

        // Rotate by 90° ccw.
        let m = rotate_radians(FRAC_PI_2);
        let mut t2 = apply_matrix(t1, m);

        // Round result to full integers
        for i in 0..3 {
            for j in 0..2 {
                t2[i][j] = t2[i][j].round();
            }
        }

        // Verify result.
        assert_eq!(t2, [
            [0.0,  0.0],
            [-5.0, 0.0],
            [0.0,  5.0],
        ]);
    }
}
