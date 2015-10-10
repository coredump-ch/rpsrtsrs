//! Shapes in the game, like units (triangles) and buildings (squares).


pub struct Unit {
    pub rotation: f64,
    pub position: [f64; 2],
    pub target: [f64; 2],
    pub selected: bool,
}


impl Unit {
    pub fn new(position: [f64;2]) -> Unit {
        println!("Create unit at {:?}", position);
        Unit {
            rotation: 0.0,
            position: position,
            target: position,
            selected: false
        }
    }
}
