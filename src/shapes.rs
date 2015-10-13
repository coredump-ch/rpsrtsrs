//! Shapes in the game, like units (triangles) and buildings (squares).
extern crate graphics;
use self::graphics::types::Triangle;
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

    /// Return the base shape of the unit.
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

    /// Calculate whether or not this unit is hit by the point at the specified position.
    pub fn is_hit(&self, position: [f64;2]) -> bool {
        let hitbox = self.get_shape();
        math::inside_triangle(hitbox, position)
    }

}


#[cfg(test)]
mod test {
    use std::f64::consts::FRAC_PI_2;
    use super::Unit;

    #[test]
    fn test_hitbox() {
        // Create a unit at position (100,100) that has been rotated by 90° CW.
        //
        //    /\
        //   /__\
        //
        let unit = Unit::new([100.0, 100.0], FRAC_PI_2);

        // The following points should be outside of the hitbox.
        assert_eq!(unit.is_hit([0.0, 0.0]), false);
        assert_eq!(unit.is_hit([unit.size, unit.size]), false);

        // The following five points should be inside the hitbox.
        //      .
        //     /.\
        //   ./_._\.
        //
        let vertical_distance = unit.size / 2.0 - 1.0;
        assert_eq!(unit.is_hit([100.0, 100.0]), true);
        assert_eq!(unit.is_hit([100.0, 100.0 - vertical_distance]), true);
        assert_eq!(unit.is_hit([100.0, 100.0 + vertical_distance]), true);
        assert_eq!(unit.is_hit([100.0 - vertical_distance, 100.0 + vertical_distance]), true);
        assert_eq!(unit.is_hit([100.0 + vertical_distance, 100.0 + vertical_distance]), true);

        // The following two points should be outside the hitbox.
        //
        //  . /\ .
        //   /__\
        //
        assert_eq!(unit.is_hit([100.0 - vertical_distance, 100.0]), false);
        assert_eq!(unit.is_hit([100.0 + vertical_distance, 100.0]), false);
    }
}
