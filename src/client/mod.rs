use std::sync::{Mutex, Arc};
use std::net::{SocketAddr, ToSocketAddrs};
use std::f64::consts::PI;
use std::collections::VecDeque;
use opengl_graphics::GlGraphics;
use opengl_graphics::glyph_cache::GlyphCache;
use piston::input::{Button, Key, MouseButton, RenderArgs, UpdateArgs};

use std::{thread, time};
use std::net::TcpStream;
use network::{Command, Message};

use bincode::{serialize_into, deserialize_from, Infinite};

use state::WorldState;
use shapes::Unit;
use colors::{BLACK, YELLOW, ORANGE};

pub mod menu;

use self::menu::Menu;

pub struct NetworkClient {
    pub world_state: Arc<Mutex<WorldState>>,
    server_addr: SocketAddr,
    stream: Option<TcpStream>,
    commands: Arc<Mutex<VecDeque<Command>>>,
}

impl NetworkClient {
    pub fn new<T: ToSocketAddrs>(server_addrs: T, world_state:
                                 Arc<Mutex<WorldState>>,
                                 commands: Arc<Mutex<VecDeque<Command>>>) -> NetworkClient {
        let server_addr = server_addrs.to_socket_addrs().unwrap().next().unwrap();
        NetworkClient {
            world_state: world_state,
            server_addr: server_addr,
            stream: None,
            commands: commands,
        }
    }

    // todo: Maybe return client_id here? Would allow the application to reconnect...
    pub fn connect(&mut self) {
        let mut stream = TcpStream::connect(self.server_addr).unwrap();
        serialize_into(&mut stream, &Message::ClientHello, Infinite).unwrap();
        let server_hello = deserialize_from(&mut stream, Infinite);

        self.stream = Some(stream);
        if let Ok(Message::ServerHello(_, world_state)) = server_hello {
            let mut world_state_lock = self.world_state.lock().unwrap();
            *world_state_lock = world_state;
        } else {
            panic!("Could not connect to server");
        }
    }

    pub fn update(&self) {
        let stream = self.stream.as_ref().expect("Stream not here :(");
        let mut command_stream = stream.try_clone().unwrap();
        let commands = self.commands.clone();

        // Command sender loop
        thread::spawn(move || {
            loop {
                let command = {
                    let mut commands = commands.lock().unwrap();
                    commands.pop_front()
                };
                command.map(|cmd| {
                    println!("Got command: {:?}", cmd);
                    //let cmd = Message::Command(Command::Move(0.into(), [100f64, 100f64]));
                    serialize_into(&mut command_stream, &Message::Command(cmd), Infinite)
                        .unwrap_or_else(|e|println!("Sending command failed: {}", e));
                });
                thread::sleep(time::Duration::from_millis(10));
            }
        });

        let mut game_state_stream = stream.try_clone().unwrap();
        let world_state = self.world_state.clone();
        thread::spawn(move || {
            loop {
                let game_state = deserialize_from(&mut game_state_stream,
                                                  Infinite);
                match game_state {
                    Ok(game) => {
                        //println!("{:?}", game);
                        let mut world_state_lock = world_state.lock().unwrap();
                        world_state_lock.game = game;

                    }
                    Err(e) => {
                        println!("{:?}", e);
                        thread::sleep(time::Duration::from_millis(200));
                    }
                }
            }
        });
    }
}

#[derive(Clone, Copy, Debug)]
pub enum State {
    Menu,
    Running,
}

pub struct App {
    pub gl: GlGraphics, // OpenGL drawing backend.
    pub world_state: Arc<Mutex<WorldState>>,
    pub units: Vec<Unit>,
    pub commands: Arc<Mutex<VecDeque<Command>>>,
    pub cursor: [f64; 2],
    pub state: State,
    menu: Menu,
}

impl App {
    pub fn new(gl: GlGraphics) -> App {
        App {
            gl: gl,
            world_state: Arc::new(Mutex::new(WorldState::new(0, 0))),
            units: vec![],
            commands: Arc::new(Mutex::new(VecDeque::new())),
            cursor: [0.0, 0.0],
            state: State::Menu,
            menu: Menu::new()
        }
    }

    pub fn start(&mut self) {
        let mut network_client = NetworkClient::new(("127.0.0.1", 8080), self.world_state.clone(), self.commands.clone());
        network_client.connect();
        network_client.update();
    }

    pub fn select(&mut self, position: [f64;2]) {
        for u in &mut self.units {
            u.selected = u.is_hit(position);
        };
    }

    fn render_game(&mut self, args: &RenderArgs, _: &mut GlyphCache) {
        use graphics::{polygon, clear};
        use graphics::Transformed;
        use graphics::types::Polygon;

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

    pub fn render(&mut self, args: &RenderArgs, cache: &mut GlyphCache) {
        match self.state {
            State::Menu => self.menu.render(args, &mut self.gl, cache), //self.render_menu(args, cache),
            State::Running => self.render_game(args, cache),
        }
    }

    pub fn update(&mut self, _: &UpdateArgs) {
        let player = {
            let world_lock = self.world_state.lock().unwrap();
            world_lock.game.players.get(0).map(|v| v.clone())
        };
        if let Some(player) = player {
            for unit in player.units.iter() {
                self.units.get_mut(unit.id.0 as usize)
                    .map(|app_unit| {
                        app_unit.position = unit.position;
                        app_unit.rotation = unit.angle;
                    })
                    .or_else(|| {
                        self.units.push(Unit::new(unit.position.clone(), unit.angle));
                        None
                    });
            }
        }
    }

    pub fn on_button_press(&mut self, button: &Button) {
        match self.state {
            State::Menu => {
                match button {
                    &Button::Keyboard(Key::Up) => {
                        self.menu.previous();
                    }
                    &Button::Keyboard(Key::Down) => {
                        self.menu.next();
                    }
                    &Button::Keyboard(Key::Return) => {
                        match self.menu.get_selected_entry() {
                            menu::Entries::Start => {
                                self.start();
                                self.state = State::Running;
                            }
                            menu::Entries::Exit => {
                            }
                        }
                    }
                    _ => { }
                }
            }
            State::Running => {
                match button {
                    &Button::Keyboard(_) => { }
                    &Button::Mouse(button) => {
                        self.on_mouse_click(&button);
                    }
                    &Button::Controller(_) => { }
                }
            }
        };
    }

    pub fn on_mouse_click(&mut self, button: &MouseButton) {
        let cursor = self.cursor;
        match *button {
            MouseButton::Left  => self.select(cursor),
            MouseButton::Right => self.move_selected(cursor),
            _ => println!("Pressed mouse button '{:?}'", button),
        }
    }

    pub fn on_mouse_move(&mut self, cursor: [f64; 2]) {
        self.cursor = cursor;
    }

    pub fn move_selected(&mut self, position: [f64;2]) {
        for i in 0..self.units.len() {
            let s = &mut self.units[i];
            if s.selected {
                s.target = position;
                let dx = position[0] - s.position[0];
                let dy = position[1] - s.position[1];
                if dx.is_sign_negative() {
                    s.rotation = (dy / dx).atan() + PI;
                } else {
                    s.rotation = (dy / dx).atan();
                }
                let id = i as u32;
                let mut commands = self.commands.lock().unwrap();
                commands.push_back(Command::Move(id.into(), position));
                println!("dx: {}, dy: {}, new rotation: {}", dx, dy, s.rotation);
            }
        }
    }
}
