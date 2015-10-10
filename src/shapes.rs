//! Shapes in the game, like triangles and squares.


pub struct Square {
    pub rotation: f64,   // Rotation for the square.
    pub position: [f64; 2],
    pub target: [f64; 2],
    pub selected: bool,
}


impl Square {
    pub fn new(position: [f64;2]) -> Square {
        println!("Create square at {:?}", position);
        Square {
            rotation: 0.0,
            position: position,
            target: position,
            selected: false
        }
    }
}
