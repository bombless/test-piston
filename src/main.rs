extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

type Frame = [[bool; 6]; 6];

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    blocks: Frame,
    last_object: Option<Frame>,
    curr_object: Frame,
    color: (u8, u8, u8),
}

trait Crash {
    fn crash(&self, rhs: &Self) -> bool;
}

impl Crash for Frame {
    fn crash(&self, rhs: &Self) -> bool {
        self[..].iter().zip(&rhs[..]).any(|(lhs, rhs)| {
            lhs[..].iter().zip(&rhs[..]).any(|(&x, &y)| x && y)
        })
    }
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        //let square = rectangle::square(0.0, 0.0, 50.0);
        let blocks = self.blocks;
        let frame = self.curr_object;
        let last_object = self.last_object;
        let color = [self.color.0 as f32 / 256.0, self.color.1 as f32 / 256.0, self.color.2 as f32 / 256.0, 1.0];

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);
            const GRAY: [f32; 4] = [0.8, 0.8, 0.8, 1.0];

            //let transform = c.transform.trans(175.0, 150.0).rot_deg(180.0);
            
            for (i, line) in blocks[..].iter().enumerate() {
                for (j, &block) in line[..].iter().enumerate() {
                    if block {
                        let transform = c.transform.trans(300.0, 0.0).rot_deg(90.0);
                        rectangle(RED, rectangle::square(i as f64 * 50.0, j as f64 * 50.0, 50.0), transform, gl);
                    }
                }
            }
            if let Some(last_object) = last_object {
                for (i, line) in last_object[..].iter().enumerate() {
                    for (j, &block) in line[..].iter().enumerate() {
                        if block {
                            let transform = c.transform.trans(300.0, 0.0).rot_deg(90.0);
                            rectangle(GRAY, rectangle::square(i as f64 * 50.0, j as f64 * 50.0, 50.0), transform, gl);
                        }
                    }
                }    
            }

            
            for (i, line) in frame[..].iter().enumerate() {
                for (j, &block) in line[..].iter().enumerate() {
                    if block {
                        let transform = c.transform.trans(300.0, 0.0).rot_deg(90.0);
                        rectangle(color, rectangle::square(i as f64 * 50.0, j as f64 * 50.0, 50.0), transform, gl);
                    }
                }
            }            
        });
    }

    fn update(&mut self, args: &UpdateArgs) -> bool {
        let crash = self.blocks.crash(&self.curr_object);
        println!("{:?}", self.last_object);
        let bottom = self.last_object.map_or(false, |o| o[5][..].iter().any(|&x| x));
        if crash || bottom {        
            if let Some(last) = self.last_object {
                extern crate rand;
                self.color = rand::random();
                self.curr_object = BlockCollection.next().unwrap();
                if self.curr_object.crash(&self.blocks) {
                    return false
                }
                self.last_object = None;
                let mut combined = Frame::default();
                for i in 0 .. 6 {
                    for j in 0 .. 6 {
                        combined[i][j] = self.blocks[i][j] || last[i][j]
                    }
                }
                self.blocks = combined;
                true
            } else {
                false
            }
        } else {
            true
        }
    }
    
    fn action(&mut self, (x0, y0): (isize, isize)) {
        use std::iter::IntoIterator;
        self.last_object = Some(self.curr_object);
        let copy = self.curr_object;
        for (i, line) in IntoIterator::into_iter(&mut self.curr_object).enumerate() {
            for (j, p) in line.into_iter().enumerate() {
                let x = x0 + i as isize;
                let y = y0 + j as isize;
                if 0 <= x && x < 6 && 0 <= y && y < 6 {
                    *p = copy[x as usize][y as usize]
                } else {
                    *p = false
                }
            }
        }
    }
}

struct BlockCollection;
impl Iterator for BlockCollection {
    type Item = Frame;
    fn next(&mut self) -> Option<Frame> {
        extern crate rand;
        const COLLECTION: [Frame; 1] = [
           [
               [false, false, true, true, false, false,],
               [false, false, true, true, false, false,],
               [false; 6],
               [false; 6],
               [false; 6],
               [false; 6],
           ]
        ];
        let count = rand::random::<usize>() % COLLECTION.len();
        Some(COLLECTION[count])
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let window: Window = WindowSettings::new(
            "game",
            [300, 300]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        blocks: [ [false; 6]; 6],
        curr_object: BlockCollection.next().unwrap(),
        last_object: None,
        color: (0, 0, 0),
    };
    
    

    let mut running = true;
    let mut waiting_action = None;
    let mut count = 0;
    
    for e in window.events() {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }
        if !running {
            continue
        }
        match e {
            Event::Input(Input::Press(Button::Keyboard(Key::Down))) => {
                waiting_action = Some((-1, 0))
            }
            Event::Input(Input::Press(Button::Keyboard(Key::Left))) => {
                waiting_action = Some((0, -1))
            }
            Event::Input(Input::Press(Button::Keyboard(Key::Right))) => {
                waiting_action = Some((0, 1))
            }
            _ => ()
        }
        if let Some(u) = e.update_args() {
            count += 1;
            if count == 200 {
                count = 0
            } else {
                continue
            }
            app.action((-1, 0));
            match waiting_action {
                Some(action) => {
                    app.action(action);
                    waiting_action = None;
                }
                _ => ()
            }
            running = app.update(&u)
        }
    }
}
