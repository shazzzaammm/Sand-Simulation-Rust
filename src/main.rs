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

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const SQUARE_SIZE: i32 = 5;
const ROWS: u32 = WIDTH / SQUARE_SIZE as u32;
const COLS: u32 = HEIGHT / SQUARE_SIZE as u32;

const BLACK: Color = Color {
    h: 0.0,
    v: 0.0,
    s: 0.0,
};
#[derive(Clone, Copy)]
struct Color {
    h: f32,
    s: f32,
    v: f32,
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        return self.as_rgba() == other.as_rgba();
    }
}

impl Color {
    fn new(hue: f32, saturation: f32, value: f32) -> Color {
        return Color {
            h: hue,
            s: saturation,
            v: value,
        };
    }

    fn as_rgba(&self) -> [f32; 4] {
        let (h, s, v) = (self.h, self.s, self.v);
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
}

struct Game {
    gl: GlGraphics,
    arr: [[Color; COLS as usize]; ROWS as usize],
    dragging: bool,
    color: Color,
}

impl Game {
    fn new(g: GlGraphics) -> Game {
        Game {
            dragging: false,
            arr: [[BLACK; COLS as usize]; ROWS as usize],
            gl: g,
            color: Color::new(255.0, 1.0, 1.0),
        }
    }

    fn render(&mut self, arg: &RenderArgs) {
        self.gl.draw(arg.viewport(), |_c, gl| {
            graphics::clear(BLACK.as_rgba(), gl);
            for i in 0..ROWS as usize {
                for j in 0..COLS as usize {
                    if self.arr[i][j] != BLACK {
                        graphics::rectangle(
                            self.arr[i][j].as_rgba(),
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

    fn update(&mut self) {
        let mut next_arr = self.arr;

        for x in 0..ROWS as usize {
            for y in 0..COLS as usize {
                // At bottom
                if y as i16 - 1 < 0 {
                    continue;
                }

                // Drop down
                if self.arr[x][y] != BLACK && self.arr[x][y - 1] == BLACK {
                    next_arr[x][y - 1] = self.arr[x][y];
                    next_arr[x][y] = BLACK;
                }
                // Go left
                else if x as i16 - 1 >= 0
                    && self.arr[x][y] != BLACK
                    && self.arr[x - 1][y - 1] == BLACK
                {
                    next_arr[x - 1][y - 1] = self.arr[x][y];
                    next_arr[x][y] = BLACK;
                }
                // Go right
                else if x as i16 + 1 < COLS as i16
                    && self.arr[x][y] != BLACK
                    && self.arr[x + 1][y - 1] == BLACK
                {
                    next_arr[x + 1][y - 1] = self.arr[x][y];
                    next_arr[x][y] = BLACK;
                }
            }
        }
        self.arr = next_arr;
    }

    fn process_mouse(&mut self, arg: [f64; 2]) {
        if !self.dragging {
            return;
        }
        let x = arg[0] as i32 / SQUARE_SIZE;
        let y = arg[1] as i32 / SQUARE_SIZE;
        if x < ROWS as i32 && y < COLS as i32 {
            self.arr[x as usize][y as usize] = self.color;
        }
        self.color.h = (self.color.h + 0.1) % 255.0;
    }

    fn process_input(&mut self, arg: &ButtonArgs) {
        match arg.state {
            ButtonState::Press => match arg.button {
                Button::Mouse(MouseButton::Left) => self.dragging = true,
                Button::Keyboard(Key::R) => self.arr = [[BLACK; COLS as usize]; ROWS as usize],
                _ => (),
            },

            ButtonState::Release => match arg.button {
                Button::Mouse(MouseButton::Left) => self.dragging = false,
                _ => (),
            },
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new("Sand?", [WIDTH, HEIGHT])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut game = Game::new(GlGraphics::new(opengl));

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
