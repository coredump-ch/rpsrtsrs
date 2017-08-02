use std::sync::{Mutex, Arc};
use std::net::{SocketAddr, ToSocketAddrs};
use std::f64::consts::PI;
use std::collections::VecDeque;
use std::error::Error;
use opengl_graphics::GlGraphics;
use opengl_graphics::glyph_cache::GlyphCache;
use piston::input::{Button, Key, MouseButton, RenderArgs, UpdateArgs};

use std::{thread, time};
use std::net::TcpStream;
use network::{Command, Message};

use bincode::{serialize_into, deserialize_from, Infinite};

use state::{ClientId, WorldState, GameState};
use shapes::Unit;
use colors::{BLACK, YELLOW, ORANGE};

pub mod menu;
pub mod error;

use self::menu::Menu;

pub struct NetworkClient {
    pub game_state: Arc<Mutex<GameState>>,
    server_addr: SocketAddr,
    stream: Option<TcpStream>,
    commands: Arc<Mutex<VecDeque<Command>>>,
}

impl NetworkClient {
    pub fn new<T: ToSocketAddrs>(server_addrs: T,
                                 game_state: Arc<Mutex<GameState>>,
                                 commands: Arc<Mutex<VecDeque<Command>>>) -> NetworkClient {
        let server_addr = server_addrs.to_socket_addrs().unwrap().next().unwrap();
        NetworkClient {
            game_state: game_state,
            server_addr: server_addr,
            stream: None,
            commands: commands,
        }
    }

    pub fn connect(&mut self) -> Result<(ClientId, WorldState), Box<Error>>  {
        let mut stream = TcpStream::connect(self.server_addr)?;
        serialize_into(&mut stream, &Message::ClientHello, Infinite)?;
        let server_hello = deserialize_from(&mut stream, Infinite);

        self.stream = Some(stream);
        if let Ok(Message::ServerHello(client_id, world_state)) = server_hello {
            Ok((client_id, world_state))
        } else {
            Err("Could not connect to server".into())
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
        let game_state = self.game_state.clone();
        thread::spawn(move || {
            loop {
                let game_state_server: Result<GameState,_> = deserialize_from(
                    &mut game_state_stream,
                    Infinite);
                match game_state_server {
                    Ok(game) => {
                        //println!("{:?}", game);
                        let mut game_state_lock = game_state.lock().unwrap();
                        *game_state_lock = game;

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

#[derive(Clone, Debug)]
pub enum State {
    Menu,
    Error(error::Message),
    Running,
}

pub struct App {
    pub gl: GlGraphics, // OpenGL drawing backend.
    pub world_state: Option<WorldState>,
    pub game_state: Arc<Mutex<GameState>>,
    pub units: Vec<Unit>,
    pub commands: Arc<Mutex<VecDeque<Command>>>,
    pub cursor: [f64; 2],
    pub state: State,
    zoom: f64,
    scroll: [f64; 2],
    menu: Menu,
    client_id: Option<ClientId>,
}

impl App {
    pub fn new(gl: GlGraphics) -> App {
        App {
            gl: gl,
            world_state: None,
            game_state: Arc::new(Mutex::new(GameState::new())),
            units: vec![],
            commands: Arc::new(Mutex::new(VecDeque::new())),
            cursor: [0.0, 0.0],
            state: State::Menu,
            zoom: 1.0,
            scroll: [0.0, 0.0],
            menu: Menu::new(),
            client_id: None,
        }
    }

    pub fn start(&mut self) -> Result<(), Box<Error>> {
        let mut network_client = NetworkClient::new(
            ("127.0.0.1", 8080),
            self.game_state.clone(),
            self.commands.clone());
        let (client_id, world_state) = network_client.connect()?;
        self.client_id = Some(client_id);
        self.world_state = Some(world_state);
        network_client.update();
        Ok(())
    }

    pub fn select(&mut self, position: [f64;2]) {
        for u in &mut self.units {
            u.selected = u.is_hit(position);
        };
    }

    fn render_game(&mut self, args: &RenderArgs, _: &mut GlyphCache) {
        use graphics::{polygon, line, clear};
        use graphics::Transformed;
        use graphics::types::{Polygon, Line};

        const FRONT_THICKNESS: f64 = 5.0;

        let units = &self.units;

        let world = self.world_state.as_ref().unwrap();
        let (wx, wy) = (world.x, world.y);
        let zoom = self.zoom;
        let scroll = self.scroll;

        self.gl.draw(args.viewport(), |c, gl| {

            let transform = c.transform
                .scale(zoom, zoom)
                .trans(scroll[0], scroll[1]);

            // Clear the screen.
            clear(BLACK, gl);

            let world: [Line; 4] = [
                [0.0, 0.0, wx,  0.0],
                [wx,  0.0, wx,  wy],
                [wx,  wy,  0.0, wy],
                [0.0, wy,  0.0, 0.0],
            ];

            for l in world.iter() {
                line(ORANGE, 1.0, *l, transform, gl);
            }

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
                let transform_front = transform.trans(s.state.position[0], s.state.position[1])
                    .rot_rad(s.state.angle)
                    .trans(-25.0, -25.0);

                // We don't need to apply any transformation to the units
                let transform_triangle = transform;

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
            State::Error(ref msg) => msg.render(args, &mut self.gl, cache),
        }
    }

    pub fn update(&mut self, _: &UpdateArgs) {
        let player = {
            let game_lock = self.game_state.lock().unwrap();
            game_lock.players.get(0).map(|v| v.clone())
        };
        if let Some(player) = player {
            for unit in player.units.iter() {
                self.units.get_mut(unit.id.0 as usize)
                    .map(|app_unit| {
                        app_unit.state = unit.clone();
                    })
                    .or_else(|| {
                        self.units.push(Unit::new(unit.clone()));
                        None
                    });
            }
        }
    }

    pub fn on_button_press(&mut self, button: &Button) -> bool {
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
                                match self.start() {
                                    Ok(_) => {
                                        self.state = State::Running;
                                    }
                                    Err(err) => {
                                        self.state = State::Error(error::Message::new(err.description().into()));
                                    }
                                }
                            }
                            menu::Entries::Exit => {
                                return true;
                            }
                        }
                    }
                    _ => { }
                }
            }
            State::Running => {
                match button {
                    &Button::Keyboard(Key::Up) => {
                        self.scroll[1] += 10.0;
                    }
                    &Button::Keyboard(Key::Down) => {
                        self.scroll[1] -= 10.0;
                    }
                    &Button::Keyboard(Key::Left) => {
                        self.scroll[0] += 10.0;
                    }
                    &Button::Keyboard(Key::Right) => {
                        self.scroll[0] -= 10.0;
                    }
                    &Button::Keyboard(_) => { }
                    &Button::Mouse(button) => {
                        self.on_mouse_click(&button);
                    }
                    &Button::Controller(_) => { }
                }
            }
            State::Error(_) => {
                match button {
                    &Button::Keyboard(_) => { self.state = State::Menu; }
                    _ => { }
                }
            }
        };
        false
    }

    pub fn on_mouse_click(&mut self, button: &MouseButton) {
        let cursor = [
            self.cursor[0] / self.zoom - self.scroll[0],
            self.cursor[1] / self.zoom - self.scroll[1],
        ];
        match *button {
            MouseButton::Left  => self.select(cursor),
            MouseButton::Right => self.move_selected(cursor),
            _ => println!("Pressed mouse button '{:?}'", button),
        }
    }

    pub fn on_mouse_move(&mut self, cursor: [f64; 2]) {
        self.cursor = cursor;
    }

    pub fn on_mouse_scroll(&mut self, scroll: [f64; 2]) {
        if scroll[1] > 0.0 {
            self.zoom *= 1.5 * scroll[1];
        } else {
            self.zoom /= 1.5 * -scroll[1];
        }
        println!("zoom: {}", self.zoom);
    }

    pub fn move_selected(&mut self, position: [f64;2]) {
        for i in 0..self.units.len() {
            let s = &mut self.units[i];
            if s.selected {
                s.target = position;
                let dx = position[0] - s.state.position[0];
                let dy = position[1] - s.state.position[1];
                if dx.is_sign_negative() {
                    s.state.angle = (dy / dx).atan() + PI;
                } else {
                    s.state.angle = (dy / dx).atan();
                }
                let id = s.state.id;
                let mut commands = self.commands.lock().unwrap();
                commands.push_back(Command::Move(id, position));
                println!("dx: {}, dy: {}, new angle: {}", dx, dy, s.state.angle);
            }
        }
    }
}
