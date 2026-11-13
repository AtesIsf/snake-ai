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
    timer: usize,
    pub max_apples: usize,
    pub max_score: f32,
}

impl Sim {
    pub fn init() -> Self {
        let fps = 60;

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

        for i in 0..n_snakes {
            apple_pos.push(Sim::generate_apple(&algo.pops[i].pos, &mut rng));
        }

        Sim { algo, rl_handle, rl_thread, gen_counter: 1, rng, n_snakes, apple_pos, fps, timer: fps as usize * 30, max_apples: 0, max_score: 0.0 }
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
                let idx = (i - 1) * 5 + (j - 1);

                // Draw Grid
                grid = Sim::get_grid(x as f32, y as f32);    
                draw_handle.draw_rectangle(x, y, 400, 400, Color::LIMEGREEN);
                for rec in grid {
                    draw_handle.draw_rectangle_lines_ex(rec, 1.0, Color::RAYWHITE);
                }
                x += 420;

                // Draw Apples
                draw_handle.draw_circle(
                    20 + self.apple_pos[idx].0 * 40 + 20 + 420 * (j - 1) as i32, 
                    140 + self.apple_pos[idx].1 * 40 + 20 + 420 * (i - 1) as i32, 
                    18.0, 
                    Color::RED
                );

                // Draw Snakes
                if !self.algo.pops[idx].is_alive {
                    continue;
                }

                for p in self.algo.pops[idx].pos.iter() {
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
        draw_handle.draw_text(
            format!("Max Apples: {}   Max Score: {:.0}", self.max_apples, self.max_score).as_str(),
            420, 50,
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

                if !s.is_alive {
                    s.on_death();
                    self.max_score = self.max_score.max(s.score);
                } else if s.pos[0] == self.apple_pos[counter] {
                    s.eat_apple();
                    self.apple_pos[counter] = Sim::generate_apple(&s.pos, &mut self.rng);
                    self.max_apples = self.max_apples.max(s.n_apples);
                }
                counter += 1;
            }
        }

        if all_dead {
            for s in self.algo.pops.iter_mut() {
                if s.is_alive {
                    s.is_alive = false;
                    s.on_death();
                    self.max_score = self.max_score.max(s.score);
                }
                self.max_apples = self.max_apples.max(s.n_apples);
            }

            self.algo.evolve();
            self.gen_counter += 1;
            self.timer = self.fps as usize * 30;

            self.apple_pos.clear();
            for i in 0..self.n_snakes {
                self.apple_pos.push(Sim::generate_apple(&self.algo.pops[i].pos, &mut self.rng));
            }
        }

        self.timer = self.timer.saturating_sub(1);
    }

    pub fn generate_apple(snake_body: &[(i32, i32)], rng: &mut ThreadRng) -> (i32, i32) {
        if snake_body.len() >= 100 {
            return (rng.gen_range(0..10), rng.gen_range(0..10));
        }
        loop {
            let pos = (rng.gen_range(0..10), rng.gen_range(0..10));
            if !snake_body.contains(&pos) {
                return pos;
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_indexing_covers_15_instances() {
        let mut indices = Vec::new();
        for i in 1..=3 {
            for j in 1..=5 {
                let idx = (i - 1) * 5 + (j - 1);
                indices.push(idx);
            }
        }
        indices.sort();
        let expected: Vec<usize> = (0..15).collect();
        assert_eq!(indices, expected);
    }

    #[test]
    fn test_get_grid_dimensions() {
        let grid = Sim::get_grid(20.0, 140.0);
        assert_eq!(grid.len(), 100);
        assert_eq!(grid[0].x, 20.0);
        assert_eq!(grid[0].y, 140.0);
        assert_eq!(grid[0].width, 40.0);
        assert_eq!(grid[0].height, 40.0);
        assert_eq!(grid[99].x, 20.0 + 9.0 * 40.0);
        assert_eq!(grid[99].y, 140.0 + 9.0 * 40.0);
    }

    #[test]
    fn test_timer_saturating_sub() {
        let mut timer: usize = 0;
        timer = timer.saturating_sub(1);
        assert_eq!(timer, 0);
    }

    #[test]
    fn test_max_apples_tracking_on_eat() {
        let mut rng = rand::thread_rng();
        let mut s1 = Snake::new(&mut rng);
        let mut s2 = Snake::new(&mut rng);

        let mut max_apples: usize = 0;

        // Snake 1 eats an apple
        s1.eat_apple();
        max_apples = max_apples.max(s1.n_apples);
        assert_eq!(s1.n_apples, 1);
        assert_eq!(max_apples, 1);

        // Snake 1 eats 3 more apples
        for _ in 0..3 {
            s1.eat_apple();
        }
        max_apples = max_apples.max(s1.n_apples);
        assert_eq!(s1.n_apples, 4);
        assert_eq!(max_apples, 4);

        // Snake 2 eats fewer apples; max_apples should remain 4
        s2.eat_apple();
        s2.eat_apple();
        max_apples = max_apples.max(s2.n_apples);
        assert_eq!(s2.n_apples, 2);
        assert_eq!(max_apples, 4);

        // Snake 2 breaks record
        for _ in 0..4 {
            s2.eat_apple();
        }
        max_apples = max_apples.max(s2.n_apples);
        assert_eq!(s2.n_apples, 6);
        assert_eq!(max_apples, 6);
    }

    #[test]
    fn test_max_score_and_apples_tracking_on_death() {
        let mut rng = rand::thread_rng();
        let mut s1 = Snake::new(&mut rng);
        let mut s2 = Snake::new(&mut rng);

        let mut max_apples: usize = 0;
        let mut max_score: f32 = 0.0;

        s1.n_apples = 2;
        s1.lifetime = 45;
        s1.on_death();

        max_score = max_score.max(s1.score);
        max_apples = max_apples.max(s1.n_apples);
        assert_eq!(max_apples, 2);
        assert!((max_score - 2000.0).abs() < f32::EPSILON);

        s2.n_apples = 5;
        s2.lifetime = 120;
        s2.on_death();

        max_score = max_score.max(s2.score);
        max_apples = max_apples.max(s2.n_apples);
        assert_eq!(max_apples, 5);
        assert!((max_score - 5000.0).abs() < f32::EPSILON);

        // Generation 2 snake with lower performance does not degrade max values
        let mut s3 = Snake::new(&mut rng);
        s3.n_apples = 1;
        s3.lifetime = 30;
        s3.on_death();

        max_score = max_score.max(s3.score);
        max_apples = max_apples.max(s3.n_apples);
        assert_eq!(max_apples, 5);
        assert!((max_score - 5000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_generate_apple_guard() {
        let mut rng = rand::thread_rng();
        let body = vec![(4, 5), (4, 6), (4, 7)];
        for _ in 0..200 {
            let apple = Sim::generate_apple(&body, &mut rng);
            assert!(apple.0 >= 0 && apple.0 < 10);
            assert!(apple.1 >= 0 && apple.1 < 10);
            assert!(!body.contains(&apple), "Generated apple must not be on snake body");
        }
    }

    #[test]
    fn test_hud_format_string() {
        let max_apples = 7;
        let max_score = 7350.0;
        let text = format!("Max Apples: {}   Max Score: {:.0}", max_apples, max_score);
        assert_eq!(text, "Max Apples: 7   Max Score: 7350");
    }
}
 
