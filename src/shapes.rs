//! Shapes in the game, like units (triangles) and buildings (squares).
extern crate graphics;
use self::graphics::types::Triangle;
use self::graphics::math;
use super::state;

use common::Vec2;


pub trait Shape {
    fn get_shape(&self, size: f64) -> Triangle;

    fn is_hit(&self, size: f64, position: Vec2) -> bool;
}

impl Shape for state::Unit {
    /// Return the base shape of the unit.
    fn get_shape(&self, size: f64) -> Triangle {
        // Base shape
        let mut triangle: Triangle = [
            [0.0, size / 2.0], // Left
            [size, size],      // Top right
            [size, 0.0],       // Bottom right
        ];

        // Transformations
        for point in triangle.iter_mut() {
            // Translate center to zero point
            *point = math::add(*point, [-size / 2.0, -size / 2.0]);

            // Rotate
            let rotation_matrix = math::rotate_radians(self.angle);
            *point = math::transform_vec(rotation_matrix, *point);

            // Translate to effective position
            *point = math::add(*point, self.position.into());
        }

        triangle
    }

    /// Calculate whether or not this unit is hit by the point at the specified position.
    fn is_hit(&self, size: f64, position: Vec2) -> bool {
        let hitbox = self.get_shape(size);
        math::inside_triangle(hitbox, position.into())
    }
}

#[cfg(test)]
mod test {
    use std::f64::consts::FRAC_PI_2;

    use super::{Shape, Vec2, state};

    #[test]
    fn test_hitbox() {
        // Create a unit at position (100,100) that has been rotated by 90° CW.
        //
        //    /\
        //   /__\
        //
        let mut unit = state::Unit::new(0, Vec2::new(100.0, 100.0));
        unit.angle = FRAC_PI_2;

        // The following points should be outside of the hitbox.
        assert_eq!(unit.is_hit(50.0, Vec2::new(0.0, 0.0)), false);
        assert_eq!(unit.is_hit(50.0, Vec2::new(50.0, 50.0)), false);

        // The following five points should be inside the hitbox.
        //      .
        //     /.\
        //   ./_._\.
        //
        let vertical_distance = 50.0 / 2.0 - 1.0;
        assert_eq!(unit.is_hit(50.0, Vec2::new(100.0, 100.0)), true);
        assert_eq!(unit.is_hit(50.0, Vec2::new(100.0, 100.0 - vertical_distance)), true);
        assert_eq!(unit.is_hit(50.0, Vec2::new(100.0, 100.0 + vertical_distance)), true);
        assert_eq!(unit.is_hit(50.0, Vec2::new(100.0 - vertical_distance, 100.0 + vertical_distance)), true);
        assert_eq!(unit.is_hit(50.0, Vec2::new(100.0 + vertical_distance, 100.0 + vertical_distance)), true);

        // The following two points should be outside the hitbox.
        //
        //  . /\ .
        //   /__\
        //
        assert_eq!(unit.is_hit(50.0, Vec2::new(100.0 - vertical_distance, 100.0)), false);
        assert_eq!(unit.is_hit(50.0, Vec2::new(100.0 + vertical_distance, 100.0)), false);
    }
}
