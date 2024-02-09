extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use rand::Rng;

/// Width of the window
const WIDTH: u32 = 1920;

/// Height of the window
const HEIGHT: u32 = 1080;

/// Size of each grain of sand
const SQUARE_SIZE: i32 = 10;

/// Grid rows
const ROWS: u32 = WIDTH / SQUARE_SIZE as u32;

/// Grid columns
const COLS: u32 = HEIGHT / SQUARE_SIZE as u32;

/// The color black
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// Converts a HSV number of (h, 1.0, 1.0) to its RGBA value  
fn to_rgba(h: f32) -> [f32; 4] {
    let (h, s, v) = (h, 1.0, 1.0);
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h >= 0.0 && h < 60.0 {
        (c, x, 0.0)
    } else if h >= 60.0 && h < 120.0 {
        (x, c, 0.0)
    } else if h >= 120.0 && h < 180.0 {
        (0.0, c, x)
    } else if h >= 180.0 && h < 240.0 {
        (0.0, x, c)
    } else if h >= 240.0 && h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    [(r + m), (g + m), (b + m), 1.0]
}

/// The sand simulation object
struct Simulation {
    gl: GlGraphics,
    grid: [[f32; COLS as usize]; ROWS as usize],
    dragging: bool,
    hue: f32,
}

impl Simulation {
    /// Create a new simulation from an instance of GlGraphics
    fn new(g: GlGraphics) -> Simulation {
        Simulation {
            dragging: false,
            grid: [[0.0; COLS as usize]; ROWS as usize],
            gl: g,
            hue: 0.01,
        }
    }

    /// Render the simulation
    fn render(&mut self, arg: &RenderArgs) {
        self.gl.draw(arg.viewport(), |_c, gl| {
            graphics::clear(BLACK, gl);
            for i in 0..ROWS as usize {
                for j in 0..COLS as usize {
                    if self.grid[i][j] != 0.0 {
                        graphics::rectangle(
                            to_rgba(self.grid[i][j]),
                            graphics::rectangle::square(
                                (i * SQUARE_SIZE as usize) as f64,
                                (j * SQUARE_SIZE as usize) as f64,
                                SQUARE_SIZE as f64,
                            ),
                            _c.transform,
                            gl,
                        );
                    }
                }
            }
        });
    }

    /// Update the simulation
    fn update(&mut self) {
        let mut next_arr = self.grid;
        let mut rand_dir: f32 = rand::thread_rng().gen();
        if rand_dir > 0.5 {
            rand_dir = -1.0;
        } else {
            rand_dir = 1.0;
        }
        let rand_dir: i16 = rand_dir as i16;

        for x in 0..ROWS as usize {
            for y in 0..COLS as usize {
                // At bottom
                if y as i16 - 1 < 0 {
                    continue;
                }

                // Drop down
                if self.grid[x][y] != 0.0 && self.grid[x][y - 1] == 0.0 {
                    next_arr[x][y - 1] = self.grid[x][y];
                    next_arr[x][y] = 0.0;
                }
                // Go one way
                else if x as i16 - rand_dir >= 0
                    && self.grid[x][y] != 0.0
                    && self.grid[(x as i16 - rand_dir) as usize][y - 1] == 0.0
                {
                    next_arr[(x as i16 - rand_dir) as usize][y - 1] = self.grid[x][y];
                    next_arr[x][y] = 0.0;
                }
                // Go the other way
                else if x as i16 + rand_dir <= COLS as i16
                    && self.grid[x][y] != 0.0
                    && self.grid[(x as i16 - rand_dir) as usize][y - 1] == 0.0
                {
                    next_arr[(x as i16 + rand_dir) as usize][y - 1] = self.grid[x][y];
                    next_arr[x][y] = 0.0;
                }
            }
        }
        self.grid = next_arr;
    }

    /// Perform functions based off the mouse state and position
    fn process_mouse(&mut self, arg: [f64; 2]) {
        if !self.dragging {
            return;
        }
        let x = arg[0] as i32 / SQUARE_SIZE;
        let y = arg[1] as i32 / SQUARE_SIZE;
        if x < ROWS as i32 && y < COLS as i32 {
            self.grid[x as usize][y as usize] = self.hue;
        }
        self.hue = (self.hue + 0.075) % 360.0 + 0.01;
    }

    /// Perform functions based off button presses
    fn process_input(&mut self, arg: &ButtonArgs) {
        match arg.state {
            ButtonState::Press => match arg.button {
                Button::Mouse(MouseButton::Left) => self.dragging = true,
                Button::Keyboard(Key::R) => self.grid = [[0.0; COLS as usize]; ROWS as usize],
                _ => (),
            },

            ButtonState::Release => match arg.button {
                Button::Mouse(MouseButton::Left) => self.dragging = false,
                _ => (),
            },
        }
    }
}

/// Driver function for the simulation
fn main() {
    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new("Sand?", [WIDTH, HEIGHT])
        .opengl(opengl)
        .exit_on_esc(true)
        .fullscreen(true)
        .build()
        .unwrap();

    let mut game = Simulation::new(GlGraphics::new(opengl));

    let e_settings = EventSettings::new();

    let mut events = Events::new(e_settings);
    let mut frame = 0;
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }
        if let Some(m) = e.mouse_cursor_args() {
            game.process_mouse(m);
        }

        if let Some(b) = e.button_args() {
            game.process_input(&b);
        }

        if frame % 5 == 0 {
            game.update();
        }
        frame += 1;
    }
}
