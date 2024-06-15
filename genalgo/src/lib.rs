use rand::{
    distributions::{
        Distribution, WeightedIndex
    }, 
    rngs::ThreadRng, Rng
};
use nnet::*;

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

#[derive(Clone)]
pub struct Snake {
    net: NNet,
    pub pos: Vec<(i32, i32)>,
    pub n_apples: usize,
    pub lifetime: usize,
    pub is_alive: bool,
    score: f32,
    dir: Direction
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
    fn new(rng: &mut ThreadRng) -> Self {
        // May change the bprint later
        let net = NNet::new(&[8, 8, 8, 4], rng);
                
        let mut pos = Vec::with_capacity(3);
        let x = 0;
        let mut y = 2;

        for _ in 0..3 {
            pos.push((x, y));
            y -= 1;
        }

        Snake { net, pos, n_apples: 0, score: 0.0, lifetime: 0, is_alive: true, dir: Direction::Down }
    }

    pub fn update(&mut self, apple_pos: (i32, i32)) {
        // nnet process

        /*
         * distance to the upper wall
         * distance to the lower wall
         * distance to the left wall
         * distance to the right wall
         * relative pos x of apple
         * relative pos y of apple
         * the direction of the snake
         * the length of the snake
         */
        let inputs = vec![
            0.0 - self.pos[0].1 as f32,
            self.pos[0].1 as f32 - 9.0,
            0.0 - self.pos[0].0 as f32,
            self.pos[0].0 as f32 - 9.0,
            apple_pos.0 as f32 - self.pos[0].0 as f32,
            apple_pos.1 as f32 - self.pos[0].1 as f32,
            self.dir.to_f32(),
            self.n_apples as f32 + 3.0 
        ];

        let response = self.net.feed_forward(inputs);
        let mut max_ind = 0;
        for i in 1..response.len() {
            if response[i] > response[max_ind] {
                max_ind = i;
            }
        }

        match max_ind {
            0 => self.dir = Direction::Up,
            1 => self.dir = Direction::Down,
            2 => self.dir = Direction::Left,
            3 => self.dir = Direction::Right,
            _ => (),
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

        // Death Check
        if ! (self.pos[0].0 <= 9 && self.pos[0].0 >= 0) {
            self.is_alive = false;
        }

        if ! (self.pos[0].1 <= 9 && self.pos[0].1 >= 0) {
            self.is_alive = false;
        }

        // Lifetime update
        self.lifetime += 1;
    }

    pub fn on_death(&mut self) {
        self.score += 10.0 * self.n_apples as f32 / (self.lifetime as f32/30.0);
        self.score += self.n_apples as f32 * 10.0;
    }

    pub fn eat_apple(&mut self) {
        self.n_apples += 1;
        self.pos.push(
            (self.pos.last().unwrap().0, self.pos.last().unwrap().1)
        );
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
        let (p1, p2) = GenAlgo::select(&self.pops, &mut self.rng);

        let mut new_gen = GenAlgo::cross(p1, p2, &mut self.rng, self.pops.len());

        GenAlgo::mutate(&mut new_gen, &mut self.rng);

        self.pops = new_gen;
    }

    fn select(snakes: &Vec<Snake>, rng: &mut ThreadRng) -> (Snake, Snake) {
        let weights: Vec<f32> = snakes.iter()
            .map(|s| f32::max(s.score, 0.1))
            .collect();

        let dist = WeightedIndex::new(weights).unwrap();

        let p1 = snakes[dist.sample(rng)].clone();
        let p2 = snakes[dist.sample(rng)].clone();
        (p1, p2)
    }

    fn cross(p1: Snake, p2: Snake, rng: &mut ThreadRng, n_snakes: usize) -> Vec<Snake> {
        let mut new_gen = Vec::with_capacity(n_snakes);
        let ps1 = p1.net.serialize();
        let ps2 = p2.net.serialize();

        for i in 0..n_snakes {
            new_gen.push(Snake::new(rng));
            let mut strand = new_gen[i].net.serialize();

            for j in 0..strand.len() {
                match rng.gen_bool(0.5) {
                    true => strand[j] = ps1[j],
                    false => strand[j] = ps2[j],
                }                
            }

            new_gen[i].net.deserialize(strand);
        }

        new_gen
    }

    fn mutate(snakes: &mut Vec<Snake>, rng: &mut ThreadRng) {
        let mut_chance = 0.5;

        let mut strands: Vec<Vec<f32>> = snakes.iter()
            .map(|s| s.net.serialize())
            .collect();

        for i in 0..strands.len() {
            for j in 0..strands[i].len() {
                match rng.gen_bool(mut_chance) {
                    true => strands[i][j] += rng.gen_range(-5.0..=5.0),
                    false => (),
                }
            }
        }

        for i in 0..snakes.len() {
            snakes[i].net.deserialize(strands[i].clone());
        }
    }
}

