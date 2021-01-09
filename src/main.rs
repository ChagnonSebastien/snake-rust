extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use std::collections::LinkedList;
use rand::random;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::*;
use piston::window::WindowSettings;

pub const SIZE: usize = 27;

#[derive(Copy, Clone)]
pub enum Directions {
    UP, DOWN, LEFT, RIGHT
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct Vector2D {
    x: usize,
    y: usize,
}

impl Vector2D {
    fn get_neighbor(&self, direction: &Directions) -> Vector2D {
        match direction {
            Directions::UP => {
                if self.y == 0 { return *self }
                Vector2D { x: self.x, y: self.y - 1 }
            },
            Directions::DOWN => {
                if self.y == SIZE - 1 { return *self }
                Vector2D { x: self.x, y: self.y + 1 }
            },
            Directions::LEFT => {
                if self.x == 0 { return *self }
                Vector2D { x: self.x - 1, y: self.y 
            }},
            Directions::RIGHT => {
                if self.x == SIZE - 1 { return *self }
                Vector2D { x: self.x + 1, y: self.y }
            },
        }
    }
}

pub struct Snake {
    body: LinkedList<Vector2D>,
    direction: Directions,
}

impl Snake {
    fn random_outside_position(&self) -> Vector2D {
        let mut outside = Vec::<Vector2D>::new();
        for i in 0..SIZE {
            for j in 0..SIZE {
                let position = Vector2D { x: i, y: j };
                if !self.body.contains(&position) {
                    outside.push(position);
                }
            }
        }
        let r = (random::<f64>() * outside.len() as f64).floor() as usize;
        outside[r]
    }
}

pub struct App {
    gl: GlGraphics,
    snake: Snake,
    next_direction: Option<Directions>,
    fruit_position: Vector2D,
    last_call: bool,
    started: bool,
    lost: bool,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const VOID: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const BACKGROUND: [f32; 4] = [0.3, 0.3, 0.35, 1.0];
        const SNAKE: [f32; 4] = [1.0, 1.0, 0.5, 1.0];
        const FRUIT: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let smaller_side = args.window_size[0].min(args.window_size[1]);
        let origin = (
            (args.window_size[0] / 2.0) - (smaller_side / 2.0),
            (args.window_size[1] / 2.0) - (smaller_side / 2.0)
        );
        let square_width = smaller_side / SIZE as f64;
        
        let square = rectangle::square(0.0, 0.0, square_width / 2.0);

        self.gl.draw(args.viewport(), |_c, gl| {
            clear(VOID, gl);
        });

        let bg_color = match self.lost {
            true => FRUIT,
            false => BACKGROUND,
        };
        self.gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform.trans(origin.0, origin.1);
            rectangle(
                bg_color,
                rectangle::square(0.0, 0.0, smaller_side),
                transform,
                gl
            );
        });

        let mut prev_pos: Option<&Vector2D> = None;
        for position in &self.snake.body {
            self.gl.draw(args.viewport(), |c, gl| {
                let transform = c
                    .transform
                    .trans(
                        (smaller_side / SIZE as f64) * position.x as f64,
                        (smaller_side / SIZE as f64) * position.y as f64
                    )
                    .trans(origin.0, origin.1)
                    .trans(square_width / 4.0, square_width / 4.0);
                rectangle(SNAKE, square, transform, gl);    
            });

            if prev_pos.is_some() {
                self.gl.draw(args.viewport(), |c, gl| {
                    let transform = c
                        .transform
                        .trans(
                            (smaller_side / SIZE as f64) * ((position.x + prev_pos.unwrap().x) as f64 / 2.0),
                            (smaller_side / SIZE as f64) * ((position.y + prev_pos.unwrap().y) as f64 / 2.0)
                        )
                        .trans(origin.0, origin.1)
                        .trans(square_width / 4.0, square_width / 4.0);
                    rectangle(SNAKE, square, transform, gl);    
                });
            }

            prev_pos = Some(position);
        }

        let fruit = self.fruit_position;
        self.gl.draw(args.viewport(), |c, gl| {
            let transform = c
                .transform
                .trans(
                    (smaller_side / SIZE as f64) * fruit.x as f64,
                    (smaller_side / SIZE as f64) * fruit.y as f64
                )
                .trans(origin.0, origin.1)
                .trans(square_width / 4.0, square_width / 4.0);
            rectangle(FRUIT, square, transform, gl);    
        });
    }

    fn update(&mut self, _args: &UpdateArgs) {

        if !self.started {
            if self.next_direction.is_none() {
                return
            } else {
                self.started = true;
            }
        }

        if self.lost {
            if self.next_direction.is_some() {
                std::process::exit(0);
            }
            return
        }

        match &self.next_direction {
            Some(direction) => {
                self.snake.direction = *direction;
                self.next_direction = None;
            },
            None => {}
        }

        let head = self.snake.body.front().expect("Snake is non existent and thus has no head");
        let eating = head.get_neighbor(&self.snake.direction);
        if self.snake.body.contains(&eating) {
            if self.last_call {
                self.lost = true
            } else {
                self.last_call = true;
            }
        } else {
            self.last_call = false;
            self.snake.body.push_front(eating);
            if eating == self.fruit_position {
                self.fruit_position = self.snake.random_outside_position();
            } else {
                self.snake.body.pop_back();
            }
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("snake", [800, 800])
        .vsync(true)
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut snake = Snake {
        body: LinkedList::new(),
        direction: Directions::UP,
    };
    snake.body.push_back( Vector2D { x: (SIZE-1)/2, y: (SIZE-1)/2 } );

    let mut app = App {
        gl: GlGraphics::new(opengl),
        fruit_position: snake.random_outside_position(),
        next_direction: None,
        snake,
        last_call: true,
        started: false,
        lost: false
    };

    let mut event_settings = EventSettings::new();
    event_settings.ups = 8;
    let mut events = Events::new(event_settings);
    while let Some(e) = events.next(&mut window) {
        if let Some(Button::Keyboard(button)) = e.press_args() {
            match button {
                Key::W | Key::Up => app.next_direction = Some(Directions::UP),
                Key::A | Key::Left => app.next_direction = Some(Directions::LEFT),
                Key::S | Key::Down => app.next_direction = Some(Directions::DOWN),
                Key::D | Key::Right => app.next_direction = Some(Directions::RIGHT),
                _ => {}
            }
        }

        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}