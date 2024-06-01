use raylib::prelude::*;
use rand::{self, rngs::ThreadRng};
use genalgo::*;

pub struct Sim {
    algo: GenAlgo,
    pub rl_handle: RaylibHandle,
    rl_thread: RaylibThread,
    gen_counter: u32,
    rng: ThreadRng,
}

impl Sim {
    pub fn init() -> Self {
        let (mut rl_handle, rl_thread) = raylib::init()
            .size(2000, 1400)
            .title("Snake Game AI")
            .build();
        rl_handle.set_exit_key(Some(KeyboardKey::KEY_NULL));
        rl_handle.set_target_fps(360);

        let algo = GenAlgo::new(rand::thread_rng(), 16);

        Sim { algo, rl_handle, rl_thread, gen_counter: 0, rng: rand::thread_rng() }
    }

    pub fn draw(&mut self) {
        let mut draw_handle = self.rl_handle.begin_drawing(&self.rl_thread);
        draw_handle.clear_background(Color::DARKGREEN);
    }

    pub fn update(&mut self) {
        todo!()
    }
}
    

