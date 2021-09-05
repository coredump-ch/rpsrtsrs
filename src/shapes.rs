//! Shapes in the game, like units (triangles) and buildings (squares).
extern crate graphics;
use self::graphics::math;
use self::graphics::math::{mul_scalar, square_len, sub};
use self::graphics::types::Triangle;
use super::state;
use std::f64;

use common::Vec2;

pub trait Shape {
    fn get_shape(&self, size: f64) -> Triangle;

    fn is_hit(&self, size: f64, position: Vec2) -> bool;

    fn collision_detect(&self, other: &Self, size: f64) -> bool;

    fn collision_avoidance(&self, other: &Self) -> (Vec2, Vec2);
}

impl Shape for state::Unit {
    /// Return the base shape of the unit.
    fn get_shape(&self, radius: f64) -> Triangle {
        // calculate side length
        let a = radius * 3.0 / 3.0f64.sqrt();

        // <|
        let mut triangle: Triangle = [
            [-radius, 0.0],           // Left
            [radius * 0.5, a / 2.0],  // Top right
            [radius * 0.5, -a / 2.0], // Bottom right
        ];

        // Transformations
        for point in triangle.iter_mut() {
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

    fn collision_detect(&self, other: &Self, size: f64) -> bool {
        let dv = sub(self.position.into(), other.position.into());
        square_len(dv) <= (2.0 * size * 2.0 * size)
    }

    fn collision_avoidance(&self, other: &Self) -> (Vec2, Vec2) {
        let d = sub(self.position.into(), other.position.into());

        let xs: [f64; 2] = self.speed_vector.into();
        let xo: [f64; 2] = other.speed_vector.into();

        let foo = (xs[0] * d[0] + xs[1] * d[1]) / (d[0] * d[0] + d[1] * d[1]);
        let ys = if foo < 0.0 {
            let c = mul_scalar(d, foo);
            sub(xs, c)
        } else {
            xs
        };

        let foo = (xo[0] * d[0] + xo[1] * d[1]) / (d[0] * d[0] + d[1] * d[1]);
        let yo = if foo > 0.0 {
            let c = mul_scalar(d, foo);
            sub(xo, c)
        } else {
            xo
        };

        return (ys.into(), yo.into());
    }
}

#[cfg(test)]
mod test {
    use std::f64::consts::FRAC_PI_2;

    use super::{state, Shape, Vec2};

    #[test]
    fn test_hitbox() {
        // Create a unit at position (100,100) that has been rotated by 90° CW.
        //
        //    /\
        //   /__\
        //
        let mut unit = state::Unit::new(0, Vec2::new(0.0, 0.0));
        unit.angle = FRAC_PI_2;

        let epsilon = 0.001;
        let size = 1.0;
        let a = size * 3.0 / 3.0f64.sqrt();

        // The following points should be outside of the hitbox.
        assert_eq!(unit.is_hit(1.0, Vec2::new(a / 2.0, epsilon)), false);
        assert_eq!(unit.is_hit(1.0, Vec2::new(a / 2.0, epsilon)), false);

        // The following five points should be inside the hitbox.
        //      .
        //     /.\
        //   ./_._\.
        //
        assert_eq!(unit.is_hit(1.0, Vec2::new(0.0, 0.0)), true);
        assert_eq!(unit.is_hit(1.0, Vec2::new(0.0, -size + epsilon)), true);
        assert_eq!(unit.is_hit(1.0, Vec2::new(0.0, size / 2.0 - epsilon)), true);
        assert_eq!(
            unit.is_hit(1.0, Vec2::new(-a / 2.0 + epsilon, size / 2.0 - epsilon)),
            true
        );
        assert_eq!(
            unit.is_hit(1.0, Vec2::new(a / 2.0 - epsilon, size / 2.0 - epsilon)),
            true
        );

        // The following two points should be outside the hitbox.
        //
        //  . /\ .
        //   /__\
        //
        assert_eq!(unit.is_hit(1.0, Vec2::new(a / 2.0, 0.0)), false);
        assert_eq!(unit.is_hit(1.0, Vec2::new(a / 2.0, 0.0)), false);
    }

    #[test]
    fn test_collision_detect() {
        let unit_l = state::Unit::new(0, Vec2::new(0.0, 0.0));

        {
            let unit_r = state::Unit::new(0, Vec2::new(1.0, 0.0));
            assert!(unit_l.collision_detect(&unit_r, 0.5));
        }

        {
            let unit_r = state::Unit::new(0, Vec2::new(1.0, 0.0));
            assert!(unit_l.collision_detect(&unit_r, 0.5));
        }
    }
}
