use raylib::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use genalgo::*;

pub struct Sim {
    algo: GenAlgo,
    pub rl_handle: RaylibHandle,
    rl_thread: RaylibThread,
    gen_counter: u32,
    rng: ThreadRng,
    apple_data: AppleData
}

struct AppleData {
    is_eaten: Vec<bool>,
    pos: Vec<(i32, i32)>
}

impl Sim {
    pub fn init() -> Self {
        let (mut rl_handle, rl_thread) = raylib::init()
            .size(2120, 1400)
            .title("Snake Game AI")
            .build();
        rl_handle.set_exit_key(Some(KeyboardKey::KEY_NULL));
        rl_handle.set_target_fps(360);

        let n_snakes = 15;
        let algo = GenAlgo::new(rand::thread_rng(), n_snakes);

        let mut rng = rand::thread_rng();
        let mut is_eaten = Vec::with_capacity(n_snakes);
        let mut pos = Vec::with_capacity(n_snakes);

        for _ in 0..n_snakes {
            is_eaten.push(false);
            pos.push(
                (rng.gen_range(0..10), rng.gen_range(0..10))
            );
        }

        let apple_data = AppleData { is_eaten, pos };

        Sim { algo, rl_handle, rl_thread, gen_counter: 0, rng, apple_data }
    }

    pub fn draw(&mut self) {
        let mut draw_handle = self.rl_handle.begin_drawing(&self.rl_thread);
        draw_handle.clear_background(Color::DARKGREEN);
        
        // Draws all the grids
        let mut x: i32;
        let mut y = 140;
        let mut grid: Vec<Rectangle>;

        for _ in 0..3 {
            x = 20;
            for _ in 0..5 {
                grid = Sim::get_grid(x as f32, y as f32);    
                draw_handle.draw_rectangle(x, y, 400, 400, Color::LIMEGREEN);
                for rec in grid {
                    draw_handle.draw_rectangle_lines_ex(rec, 1.0, Color::RAYWHITE);
                }
                x += 420;
            }
            y += 420;
        }
        
        // Draw Apples
        for i in 1..=3 {
            for j in 1..=5 {
                // It's ugly but it works
                draw_handle.draw_circle(
                    20 + self.apple_data.pos[i * j - 1].0 * 40 + 20 + 420 * (j - 1) as i32, 
                    140 + self.apple_data.pos[i * j - 1].1 * 40 + 20 + 420 * (i - 1) as i32, 
                    18.0, 
                    Color::RED
                );
            }
        }

        // Draws FPS & other info
        draw_handle.draw_fps(20, 10);
    }

    pub fn update(&mut self) {
        todo!()
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
    

