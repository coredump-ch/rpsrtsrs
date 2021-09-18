use std::collections::VecDeque;
use std::error::Error;
use std::mem;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use bincode::Options;
use opengl_graphics::GlGraphics;
use opengl_graphics::GlyphCache;
use piston::input::{Button, Key, MouseButton, RenderArgs, UpdateArgs};

use colors::{self, BLACK, ORANGE, TRANSPARENT_WHITE, WHITE};
use common::Vec2;
use network::{Command, Message};
use shapes::Shape;
use state::{ClientId, GameState, UnitId, WorldState, UNIT_SIZE};

pub mod error;
pub mod menu;

use self::menu::Menu;

pub struct NetworkClient {
    pub game_state: Arc<Mutex<Option<GameState>>>,
    server_addr: SocketAddr,
    stream: Option<TcpStream>,
    commands: Arc<Mutex<VecDeque<Command>>>,
}

impl NetworkClient {
    pub fn new<T: ToSocketAddrs>(
        server_addrs: T,
        game_state: Arc<Mutex<Option<GameState>>>,
        commands: Arc<Mutex<VecDeque<Command>>>,
    ) -> NetworkClient {
        let server_addr = server_addrs.to_socket_addrs().unwrap().next().unwrap();
        NetworkClient {
            game_state,
            server_addr,
            stream: None,
            commands,
        }
    }

    pub fn connect(&mut self) -> Result<(ClientId, WorldState), Box<dyn Error>> {
        let mut stream = TcpStream::connect(self.server_addr)?;
        let bincode = bincode::DefaultOptions::new().with_limit(1024);
        println!("Sending client hello");
        bincode.serialize_into(&mut stream, &Message::ClientHello)?;
        let server_hello = bincode.deserialize_from(&mut stream);

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
        let bincode = bincode::DefaultOptions::new().with_limit(1024);

        // Command sender loop
        thread::spawn(move || {
            loop {
                let command = {
                    let mut commands = commands.lock().unwrap();
                    commands.pop_front()
                };
                if let Some(cmd) = command {
                    println!("Got command: {:?}", cmd);
                    //let cmd = Message::Command(Command::Move(0.into(), [100f64, 100f64]));
                    bincode
                        .serialize_into(&mut command_stream, &Message::Command(cmd))
                        .unwrap_or_else(|e| println!("Sending command failed: {}", e));
                }
                thread::sleep(time::Duration::from_millis(10));
            }
        });

        let mut game_state_stream = stream.try_clone().unwrap();
        let game_state = self.game_state.clone();
        thread::spawn(move || {
            loop {
                let game_state_server: Result<GameState, _> =
                    bincode.deserialize_from(&mut game_state_stream);
                match game_state_server {
                    Ok(game) => {
                        //println!("{:?}", game);
                        let mut game_state_lock = game_state.lock().unwrap();
                        *game_state_lock = Some(game);
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
    pub game_state_server: Arc<Mutex<Option<GameState>>>,
    pub game_state: GameState,
    pub selected_units: Vec<UnitId>,
    pub commands: Arc<Mutex<VecDeque<Command>>>,
    pub cursor: Vec2,
    pub state: State,
    zoom: f64,
    scroll: Vec2,
    menu: Menu,
    client_id: Option<ClientId>,
    server_ip: String,
    server_port: u16,
    debug: bool,
}

impl App {
    pub fn new(gl: GlGraphics, server_ip: String, server_port: u16) -> App {
        App {
            gl,
            world_state: None,
            game_state_server: Arc::new(Mutex::new(None)),
            game_state: GameState::new(),
            selected_units: vec![],
            commands: Arc::new(Mutex::new(VecDeque::new())),
            cursor: Vec2::new(0.0, 0.0),
            state: State::Menu,
            zoom: 1.0,
            scroll: Vec2::new(0.0, 0.0),
            menu: Menu::new(),
            client_id: None,
            server_ip,
            server_port,
            debug: false,
        }
    }

    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let mut network_client = NetworkClient::new(
            (&*self.server_ip, self.server_port),
            self.game_state_server.clone(),
            self.commands.clone(),
        );
        let (client_id, world_state) = network_client.connect()?;
        self.client_id = Some(client_id);
        self.world_state = Some(world_state);
        network_client.update();
        Ok(())
    }

    pub fn select(&mut self, position: Vec2) {
        let player = {
            let index = self.client_id.unwrap_or(ClientId(0)).0 as usize;
            self.game_state.players.get(index)
        };

        self.selected_units.truncate(0);
        if let Some(player) = player {
            for unit in player.units.iter() {
                if unit.is_hit(UNIT_SIZE, position) {
                    self.selected_units.push(unit.id);
                }
            }
        }
    }

    fn render_game(&mut self, args: &RenderArgs, _: &mut GlyphCache) {
        use graphics::types::{Line, Polygon};
        use graphics::Transformed;
        use graphics::{clear, ellipse, line, polygon};

        const FRONT_THICKNESS: f64 = 5.0;

        let game_state = &self.game_state;
        let world = self.world_state.as_ref().unwrap();
        let (wx, wy) = (world.x, world.y);
        let zoom = self.zoom;
        let scroll = self.scroll;
        let selected_units = self.selected_units.clone();
        let debug = self.debug;

        self.gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform.scale(zoom, zoom).trans(scroll.x, scroll.y);

            // Clear the screen.
            clear(BLACK, gl);

            let world: [Line; 4] = [
                [0.0, 0.0, wx, 0.0],
                [wx, 0.0, wx, wy],
                [wx, wy, 0.0, wy],
                [0.0, wy, 0.0, 0.0],
            ];

            for l in world.iter() {
                line(ORANGE, 1.0, *l, transform, gl);
            }

            for i in 0..game_state.players.len() {
                let player = &game_state.players[i];
                let color = &colors::PLAYERS[i % colors::PLAYERS.len()];
                for s in player.units.iter() {
                    // Create a triangle polygon. The initial orientation is facing east.
                    let triangle: Polygon = &s.get_shape(UNIT_SIZE);

                    // Create a border on the front of the polygon. This is a trapezoid.
                    // Because the angle of the trapezoid side is 22.5°, we know that `dx` is always `2 * dy`.
                    let front: Polygon = &[
                        [UNIT_SIZE, UNIT_SIZE],                               // Top right
                        [UNIT_SIZE, 0.0],                                     // Bottom right
                        [UNIT_SIZE - FRONT_THICKNESS, FRONT_THICKNESS / 2.0], // Bottom left
                        [
                            UNIT_SIZE - FRONT_THICKNESS,
                            UNIT_SIZE - FRONT_THICKNESS / 2.0,
                        ], // Top left
                    ];

                    // Rotate the front to match the unit
                    let transform_front = transform
                        .trans(s.position.x, s.position.y)
                        .rot_rad(s.angle)
                        .trans(-0.5 * UNIT_SIZE, -0.5 * UNIT_SIZE);

                    let transform_circle = transform.trans(s.position[0], s.position[1]);

                    // We don't need to apply any transformation to the units
                    let transform_triangle = transform;

                    // Draw the unit ORANGE if selected
                    let selected = selected_units.iter().any(|id| id == &s.id);
                    if selected {
                        polygon(color.primary, triangle, transform_triangle, gl);
                        polygon(color.secondary, front, transform_front, gl);
                    } else {
                        polygon(color.secondary, triangle, transform_triangle, gl);
                        polygon(color.primary, front, transform_front, gl);
                    }
                    if debug {
                        ellipse(
                            TRANSPARENT_WHITE,
                            [-UNIT_SIZE, -UNIT_SIZE, 2.0 * UNIT_SIZE, 2.0 * UNIT_SIZE],
                            transform_circle,
                            gl,
                        );
                    }
                }
            }
            for b in game_state.bullets.iter() {
                let transform = transform.trans(b.position.x, b.position.y);
                ellipse(WHITE, [0.0, 0.0, 1.0, 1.0], transform, gl);
            }
        });
    }

    pub fn render(&mut self, args: &RenderArgs, cache: &mut GlyphCache) {
        match self.state {
            State::Menu => self.menu.render(args, &mut self.gl, cache),
            State::Running => self.render_game(args, cache),
            State::Error(ref msg) => msg.render(args, &mut self.gl, cache),
        }
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        // grab updated server state if it is available
        let game_state_option = {
            let mut game_state_lock = self.game_state_server.lock().unwrap();
            mem::replace(&mut *game_state_lock, None)
        };
        if let Some(game_state) = game_state_option {
            self.game_state = game_state;
        } else if let Some(ref world) = self.world_state {
            self.game_state.update(world, args.dt * 1000.0);
        }
    }

    pub fn on_button_press(&mut self, button: &Button) -> bool {
        match self.state {
            State::Menu => match button {
                Button::Keyboard(Key::Up) => {
                    self.menu.previous();
                }
                Button::Keyboard(Key::Down) => {
                    self.menu.next();
                }
                Button::Keyboard(Key::Return) => match self.menu.get_selected_entry() {
                    menu::Entries::Start => match self.start() {
                        Ok(_) => {
                            self.state = State::Running;
                        }
                        Err(err) => {
                            self.state = State::Error(error::Message::new(err.to_string()));
                        }
                    },
                    menu::Entries::Exit => {
                        return true;
                    }
                },
                _ => {}
            },
            State::Running => match button {
                Button::Keyboard(Key::Up) => {
                    self.scroll.y += 10.0;
                }
                Button::Keyboard(Key::Down) => {
                    self.scroll.y -= 10.0;
                }
                Button::Keyboard(Key::Left) => {
                    self.scroll.x += 10.0;
                }
                Button::Keyboard(Key::Right) => {
                    self.scroll.x -= 10.0;
                }
                Button::Keyboard(Key::F) => {
                    self.shoot();
                }
                Button::Keyboard(Key::D) => {
                    self.debug = !self.debug;
                }
                Button::Keyboard(_) => {}
                Button::Mouse(button) => {
                    self.on_mouse_click(button);
                }
                Button::Controller(_) => {}
            },
            State::Error(_) => {
                if let Button::Keyboard(_) = button {
                    self.state = State::Menu;
                }
            }
        };
        false
    }

    pub fn on_mouse_click(&mut self, button: &MouseButton) {
        let cursor = self.cursor / self.zoom - self.scroll;
        match *button {
            MouseButton::Left => self.select(cursor),
            MouseButton::Right => self.move_selected(cursor),
            _ => println!("Pressed mouse button '{:?}'", button),
        }
    }

    pub fn on_mouse_move(&mut self, cursor: Vec2) {
        self.cursor = cursor;
    }

    pub fn on_mouse_scroll(&mut self, scroll: Vec2) {
        if scroll.y > 0.0 {
            self.zoom *= 1.5 * scroll.y;
        } else {
            self.zoom /= 1.5 * -scroll.y;
        }
        println!("zoom: {}", self.zoom);
    }

    pub fn move_selected(&mut self, position: Vec2) {
        for u in self.selected_units.iter() {
            let mut commands = self.commands.lock().unwrap();
            commands.push_back(Command::Move(*u, position));
        }
    }

    pub fn shoot(&mut self) {
        for u in self.selected_units.iter() {
            let mut commands = self.commands.lock().unwrap();
            commands.push_back(Command::Shoot(*u));
        }
    }
}
