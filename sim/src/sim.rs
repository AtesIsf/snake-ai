use raylib::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use genalgo::*;

pub struct Sim {
    algo: GenAlgo,
    pub rl_handle: RaylibHandle,
    rl_thread: RaylibThread,
    gen_counter: usize,
    rng: ThreadRng,
    n_snakes: usize,
    apple_pos: Vec<(i32, i32)>,
    fps: u32,
    timer: usize
}

impl Sim {
    pub fn init() -> Self {
        let fps = 30;

        let (mut rl_handle, rl_thread) = raylib::init()
            .size(2120, 1400)
            .title("Snake Game AI")
            .build();
        rl_handle.set_exit_key(Some(KeyboardKey::KEY_NULL));
        rl_handle.set_target_fps(fps);

        let n_snakes = 15;
        let algo = GenAlgo::new(rand::thread_rng(), n_snakes);

        let mut rng = rand::thread_rng();
        let mut apple_pos = Vec::with_capacity(n_snakes);

        for _ in 0..n_snakes {
            apple_pos.push(
                (rng.gen_range(0..10), rng.gen_range(0..10))
            );
        }

        Sim { algo, rl_handle, rl_thread, gen_counter: 1, rng, n_snakes, apple_pos, fps, timer: fps as usize * 30 }
    }

    pub fn draw(&mut self) {
        let mut draw_handle = self.rl_handle.begin_drawing(&self.rl_thread);
        draw_handle.clear_background(Color::DARKGREEN);
        
        // Draws all the grids
        let mut x: i32;
        let mut y = 140;
        let mut grid: Vec<Rectangle>;

        for i in 1..=3 {
            x = 20;
            for j in 1..=5 {
                // Draw Grid
                grid = Sim::get_grid(x as f32, y as f32);    
                draw_handle.draw_rectangle(x, y, 400, 400, Color::LIMEGREEN);
                for rec in grid {
                    draw_handle.draw_rectangle_lines_ex(rec, 1.0, Color::RAYWHITE);
                }
                x += 420;

                // Draw Apples
                draw_handle.draw_circle(
                    20 + self.apple_pos[i * j - 1].0 * 40 + 20 + 420 * (j - 1) as i32, 
                    140 + self.apple_pos[i * j - 1].1 * 40 + 20 + 420 * (i - 1) as i32, 
                    18.0, 
                    Color::RED
                );

                // Draw Snakes
                if !self.algo.pops[i * j - 1].is_alive {
                    continue;
                }

                for p in self.algo.pops[i * j - 1].pos.iter() {
                    draw_handle.draw_circle(
                        20 + p.0 * 40 + 20 + 420 * (j - 1) as i32, 
                        140 + p.1 * 40 + 20 + 420 * (i - 1) as i32, 
                        20.0, 
                        Color::AZURE
                    );
                }
            }
            y += 420;
        }
       
        // Draws FPS & other info
        draw_handle.draw_fps(20, 10);
        draw_handle.draw_text(
            format!("Generation: {}", self.gen_counter).as_str(),
            20, 50, 
            40, 
            Color::RAYWHITE
        );
    }

    pub fn update(&mut self) {
        let mut counter = 0;
        let mut all_dead = true;

        if self.timer > 0 {
            for s in self.algo.pops.iter_mut() {

                if !s.is_alive {
                    counter += 1;
                    continue;
                }

                all_dead = false;
                s.update(self.apple_pos[counter]);

                if s.pos[0] == self.apple_pos[counter] {
                    self.apple_pos[counter] = (self.rng.gen_range(0..10), self.rng.gen_range(0..10));
                    s.eat_apple();
                }
                counter += 1;
            }
        }

        if all_dead {
            for s in self.algo.pops.iter_mut() {
                s.on_death();
            }

            self.algo.evolve();
            self.gen_counter += 1;
            self.timer = self.fps as usize * 30;

            self.apple_pos.clear();
            for _ in 0..self.n_snakes {
                self.apple_pos.push(
                    (self.rng.gen_range(0..10), self.rng.gen_range(0..10))
                );
            }
        }

        self.timer -= 1;
    }

    fn get_grid(x: f32, y: f32) -> Vec<Rectangle> {
        let mut grid =  Vec::with_capacity(100);

        let mut curr_x: f32; 
        let mut curr_y = y;

        for _ in 0..10 {
            curr_x = x;

            for _ in 0..10 {
                grid.push(Rectangle { x: curr_x, y: curr_y, width: 40.0, height: 40.0 });
                curr_x += 40.0;
            }

            curr_y += 40.0;
        }
        
        grid
    }
}
 
