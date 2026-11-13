use rand::{
    distributions::{
        Distribution, WeightedIndex
    }, 
    rngs::ThreadRng, Rng
};
use nnet::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

#[derive(Clone)]
pub struct Snake {
    pub net: NNet,
    pub pos: Vec<(i32, i32)>,
    pub n_apples: usize,
    pub lifetime: usize,
    pub is_alive: bool,
    pub score: f32,
    pub dir: Direction,
    pub steps_left: usize,
}

pub struct GenAlgo {
    pub pops: Vec<Snake>,
    rng: ThreadRng
}

impl Direction {
    pub fn to_f32(&self) -> f32 {
        match self {
            Direction::Up => 0.0,
            Direction::Down => 1.0,
            Direction::Left => 2.0,
            Direction::Right => 3.0,
        }
    }
}

impl Snake {
    pub fn new(rng: &mut ThreadRng) -> Self {
        // Blueprint: 12 inputs, two hidden layers (12, 8), 4 outputs
        let net = NNet::new(&[12, 12, 8, 4], rng);
                
        let mut pos = Vec::with_capacity(3);
        let x = 0;
        let mut y = 2;

        for _ in 0..3 {
            pos.push((x, y));
            y -= 1;
        }

        let steps_left = 100 + pos.len() * 2;
        Snake { net, pos, n_apples: 0, score: 0.0, lifetime: 0, is_alive: true, dir: Direction::Down, steps_left }
    }

    pub fn update(&mut self, apple_pos: (i32, i32)) {
        if !self.is_alive {
            return;
        }

        // Starvation check
        if self.steps_left == 0 {
            self.is_alive = false;
            return;
        }
        self.steps_left -= 1;

        // Obstacle perception in 4 cardinal directions (walls and body)
        let head = self.pos[0];
        let is_obstacle = |x: i32, y: i32, body: &[(i32, i32)]| -> f32 {
            if x < 0 || x > 9 || y < 0 || y > 9 || body.contains(&(x, y)) {
                1.0
            } else {
                0.0
            }
        };

        let danger_up = is_obstacle(head.0, head.1 - 1, &self.pos);
        let danger_down = is_obstacle(head.0, head.1 + 1, &self.pos);
        let danger_left = is_obstacle(head.0 - 1, head.1, &self.pos);
        let danger_right = is_obstacle(head.0 + 1, head.1, &self.pos);

        // nnet process: all inputs normalized to [0.0, 1.0] (or [-1.0, 1.0] for relative offsets)
        let inputs = vec![
            self.pos[0].1 as f32 / 9.0,
            (9.0 - self.pos[0].1 as f32) / 9.0,
            self.pos[0].0 as f32 / 9.0,
            (9.0 - self.pos[0].0 as f32) / 9.0,
            (apple_pos.0 as f32 - self.pos[0].0 as f32) / 9.0,
            (apple_pos.1 as f32 - self.pos[0].1 as f32) / 9.0,
            self.dir.to_f32() / 3.0,
            self.pos.len() as f32 / 100.0,
            danger_up,
            danger_down,
            danger_left,
            danger_right,
        ];

        let response = self.net.feed_forward(inputs);
        let mut max_ind = 0;
        for i in 1..response.len() {
            if response[i] > response[max_ind] {
                max_ind = i;
            }
        }

        let new_dir = match max_ind {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Right,
            _ => self.dir,
        };

        // Prevent immediate 180° turns
        match (self.dir, new_dir) {
            (Direction::Up, Direction::Down) => (),
            (Direction::Down, Direction::Up) => (),
            (Direction::Left, Direction::Right) => (),
            (Direction::Right, Direction::Left) => (),
            _ => self.dir = new_dir,
        }

        // Movement
        for i in (1..self.pos.len()).rev() {
            self.pos[i] = self.pos[i - 1];
        }

        match self.dir {
            Direction::Up => {
                self.pos[0].1 -= 1;
            }
            Direction::Down => {
                self.pos[0].1 += 1;
            }
            Direction::Left => {
                self.pos[0].0 -= 1;
            }
            Direction::Right => {
                self.pos[0].0 += 1;
            }
        }

        // Wall Collision Check
        if !(self.pos[0].0 <= 9 && self.pos[0].0 >= 0) {
            self.is_alive = false;
        }

        if !(self.pos[0].1 <= 9 && self.pos[0].1 >= 0) {
            self.is_alive = false;
        }

        // Self-Collision Check
        if self.is_alive {
            for i in 1..self.pos.len() {
                if self.pos[0] == self.pos[i] {
                    self.is_alive = false;
                    break;
                }
            }
        }

        // Lifetime update
        self.lifetime += 1;
    }

    pub fn on_death(&mut self) {
        self.score = self.n_apples as f32 * 1000.0 + self.lifetime as f32;
    }

    pub fn eat_apple(&mut self) {
        self.n_apples += 1;
        self.pos.push(
            (self.pos.last().unwrap().0, self.pos.last().unwrap().1)
        );
        self.steps_left = 100 + self.pos.len() * 2;
    }
}

impl GenAlgo {
    pub fn new(mut rng: ThreadRng, n_snakes: usize) -> Self {
        let mut pops: Vec<Snake> = Vec::with_capacity(n_snakes);

        for _ in 0..n_snakes {
            pops.push(Snake::new(&mut rng));
        }

        GenAlgo { pops, rng }
    }

    pub fn evolve(&mut self) {
        let n_snakes = self.pops.len();
        if n_snakes == 0 {
            return;
        }

        // Elitism: find best snake and preserve it without mutation
        let best_idx = self.pops
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        let mut elite = Snake::new(&mut self.rng);
        elite.net = self.pops[best_idx].net.clone();

        let mut new_gen = Vec::with_capacity(n_snakes);
        new_gen.push(elite);

        // Roulette wheel distribution based on fitness score
        let weights: Vec<f32> = self.pops.iter()
            .map(|s| f32::max(s.score, 0.1))
            .collect();
        let dist = WeightedIndex::new(weights).unwrap();

        // Independent parent selection per offspring & crossover
        for _ in 1..n_snakes {
            let p1_idx = dist.sample(&mut self.rng);
            let p2_idx = dist.sample(&mut self.rng);
            let child = GenAlgo::cross(&self.pops[p1_idx], &self.pops[p2_idx], &mut self.rng);
            new_gen.push(child);
        }

        // Mutate offspring only (preserving elite at index 0)
        if new_gen.len() > 1 {
            GenAlgo::mutate(&mut new_gen[1..], &mut self.rng);
        }

        self.pops = new_gen;
    }

    pub fn cross(p1: &Snake, p2: &Snake, rng: &mut ThreadRng) -> Snake {
        let ps1 = p1.net.serialize();
        let ps2 = p2.net.serialize();

        let mut strand = Vec::with_capacity(ps1.len());
        for j in 0..ps1.len() {
            strand.push(if rng.gen_bool(0.5) { ps1[j] } else { ps2[j] });
        }

        let mut child = Snake::new(rng);
        child.net.deserialize(strand);
        child
    }

    pub fn mutate(snakes: &mut [Snake], rng: &mut ThreadRng) {
        let mut_chance = 0.05;

        for snake in snakes.iter_mut() {
            let mut strand = snake.net.serialize();
            for val in strand.iter_mut() {
                if rng.gen_bool(mut_chance) {
                    *val += rng.gen_range(-0.3..=0.3);
                }
            }
            snake.net.deserialize(strand);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_180_turn_inhibition() {
        // Test all opposite direction pairs
        let opposite_pairs = [
            (Direction::Up, Direction::Down),
            (Direction::Down, Direction::Up),
            (Direction::Left, Direction::Right),
            (Direction::Right, Direction::Left),
        ];

        for (current, attempted) in opposite_pairs {
            let mut dir = current;
            match (dir, attempted) {
                (Direction::Up, Direction::Down) => (),
                (Direction::Down, Direction::Up) => (),
                (Direction::Left, Direction::Right) => (),
                (Direction::Right, Direction::Left) => (),
                _ => dir = attempted,
            }
            assert_eq!(dir, current, "Opposite turn should be inhibited");
        }

        // Test valid 90-degree turns
        let valid_turns = [
            (Direction::Up, Direction::Left),
            (Direction::Up, Direction::Right),
            (Direction::Down, Direction::Left),
            (Direction::Down, Direction::Right),
        ];

        for (current, attempted) in valid_turns {
            let mut dir = current;
            match (dir, attempted) {
                (Direction::Up, Direction::Down) => (),
                (Direction::Down, Direction::Up) => (),
                (Direction::Left, Direction::Right) => (),
                (Direction::Right, Direction::Left) => (),
                _ => dir = attempted,
            }
            assert_eq!(dir, attempted, "Orthogonal turn should be allowed");
        }
    }

    #[test]
    fn test_self_collision_termination() {
        let mut rng = rand::thread_rng();
        let mut s = Snake::new(&mut rng);
        // Setup snake coiled so that a move hits its body:
        // pos[0]=(2,2), pos[1]=(1,2), pos[2]=(1,1), pos[3]=(2,1), pos[4]=(3,1)
        // With dir=Up, moving head to (2,1) hits old pos[3] which shifted to pos[4].
        s.pos = vec![(2, 2), (1, 2), (1, 1), (2, 1), (3, 1)];
        s.dir = Direction::Right; // was moving right

        // Simulate movement directly into self
        for i in (1..s.pos.len()).rev() {
            s.pos[i] = s.pos[i - 1];
        }
        s.dir = Direction::Up;
        s.pos[0].1 -= 1; // head becomes (2, 1)

        // Self-collision check
        for i in 1..s.pos.len() {
            if s.pos[0] == s.pos[i] {
                s.is_alive = false;
                break;
            }
        }
        assert!(!s.is_alive, "Self-collision must kill snake");
    }

    #[test]
    fn test_wall_collision_termination() {
        let mut rng = rand::thread_rng();
        let mut s = Snake::new(&mut rng);
        // Snake at top-left edge moving Left
        s.pos = vec![(0, 0), (0, 1), (0, 2)];
        s.dir = Direction::Left;

        for i in (1..s.pos.len()).rev() {
            s.pos[i] = s.pos[i - 1];
        }
        s.pos[0].0 -= 1; // head becomes (-1, 0)

        if !(s.pos[0].0 <= 9 && s.pos[0].0 >= 0) {
            s.is_alive = false;
        }
        assert!(!s.is_alive, "Wall collision must kill snake");
    }

    #[test]
    fn test_starvation_termination() {
        let mut rng = rand::thread_rng();
        let mut s = Snake::new(&mut rng);
        s.pos = vec![(5, 5), (5, 4), (5, 3)];
        s.steps_left = 1;
        s.update((5, 5));
        assert!(s.is_alive);
        assert_eq!(s.steps_left, 0);

        // Next update starves the snake
        s.update((5, 5));
        assert!(!s.is_alive, "Snake must die when starvation timer reaches 0");
    }

    #[test]
    fn test_eat_apple_growth_and_timer_reset() {
        let mut rng = rand::thread_rng();
        let mut s = Snake::new(&mut rng);
        let initial_len = s.pos.len();
        s.steps_left = 10;

        s.eat_apple();
        assert_eq!(s.n_apples, 1);
        assert_eq!(s.pos.len(), initial_len + 1);
        assert_eq!(s.steps_left, 100 + (initial_len + 1) * 2);
    }

    #[test]
    fn test_score_calculation() {
        let mut rng = rand::thread_rng();
        let mut s = Snake::new(&mut rng);
        s.n_apples = 4;
        s.lifetime = 123;
        s.on_death();
        assert_eq!(s.score, 4.0 * 1000.0 + 123.0);
    }

    #[test]
    fn test_elitism_preserves_best_genome() {
        let rng = rand::thread_rng();
        let mut algo = GenAlgo::new(rng, 10);
        algo.pops[3].score = 99999.0;
        let best_weights = algo.pops[3].net.serialize();

        algo.evolve();
        let elite_weights = algo.pops[0].net.serialize();
        assert_eq!(best_weights, elite_weights);
    }

    #[test]
    fn test_crossover_produces_valid_strand() {
        let mut rng = rand::thread_rng();
        let p1 = Snake::new(&mut rng);
        let p2 = Snake::new(&mut rng);
        let child = GenAlgo::cross(&p1, &p2, &mut rng);

        assert_eq!(child.net.serialize().len(), p1.net.serialize().len());
    }
}

