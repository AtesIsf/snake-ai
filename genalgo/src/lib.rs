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
    pub proximity_score: f32,
    pub dir: Direction,
    pub steps_left: usize,
}

pub struct GenAlgo {
    pub pops: Vec<Snake>,
    rng: ThreadRng
}

impl Direction {
    pub fn turn_left(&self) -> Direction {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }

    pub fn turn_right(&self) -> Direction {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }
}

impl Snake {
    pub fn new(rng: &mut ThreadRng) -> Self {
        // Blueprint: 7 inputs, two hidden layers (12, 8), 3 outputs
        let net = NNet::new(&[7, 12, 8, 3], rng);
        let pos = vec![(4, 5), (4, 6), (4, 7)];
        let steps_left = 100 + pos.len() * 2;
        Snake {
            net,
            pos,
            n_apples: 0,
            lifetime: 0,
            is_alive: true,
            score: 0.0,
            proximity_score: 0.0,
            dir: Direction::Up,
            steps_left,
        }
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

        let head = self.pos[0];
        let old_dist = (head.0 - apple_pos.0).abs() + (head.1 - apple_pos.1).abs();

        let is_obstacle = |x: i32, y: i32, body: &[(i32, i32)]| -> f32 {
            if x < 0 || x > 9 || y < 0 || y > 9 || body.contains(&(x, y)) {
                1.0
            } else {
                0.0
            }
        };

        let step_dir = |pos: (i32, i32), dir: Direction| -> (i32, i32) {
            match dir {
                Direction::Up => (pos.0, pos.1 - 1),
                Direction::Down => (pos.0, pos.1 + 1),
                Direction::Left => (pos.0 - 1, pos.1),
                Direction::Right => (pos.0 + 1, pos.1),
            }
        };

        // Egocentric sensory inputs:
        // 1. Danger 1 step ahead, left, right
        // The tail segment vacates on the next move, so it is not an immediate obstacle.
        let body_obstacles = if self.pos.len() > 1 {
            &self.pos[..self.pos.len() - 1]
        } else {
            &self.pos[..]
        };

        let pos_ahead = step_dir(head, self.dir);
        let pos_left = step_dir(head, self.dir.turn_left());
        let pos_right = step_dir(head, self.dir.turn_right());

        let danger_ahead = is_obstacle(pos_ahead.0, pos_ahead.1, body_obstacles);
        let danger_left = is_obstacle(pos_left.0, pos_left.1, body_obstacles);
        let danger_right = is_obstacle(pos_right.0, pos_right.1, body_obstacles);

        // 2. Apple orientation relative to heading: forward and lateral alignment
        let (forward_delta, lateral_delta) = match self.dir {
            Direction::Up => (head.1 - apple_pos.1, apple_pos.0 - head.0),
            Direction::Down => (apple_pos.1 - head.1, head.0 - apple_pos.0),
            Direction::Left => (head.0 - apple_pos.0, head.1 - apple_pos.1),
            Direction::Right => (apple_pos.0 - head.0, apple_pos.1 - head.1),
        };

        let forward_align = if forward_delta > 0 {
            1.0
        } else if forward_delta < 0 {
            -1.0
        } else {
            0.0
        };

        let lateral_align = if lateral_delta > 0 {
            1.0
        } else if lateral_delta < 0 {
            -1.0
        } else {
            0.0
        };

        // 3. Normalized distance to apple
        let dx = apple_pos.0 - head.0;
        let dy = apple_pos.1 - head.1;
        let norm_dist = (dx.abs() + dy.abs()) as f32 / 18.0;

        // 4. Normalized snake length
        let norm_len = self.pos.len() as f32 / 100.0;

        let inputs = vec![
            danger_ahead,
            danger_left,
            danger_right,
            forward_align,
            lateral_align,
            norm_dist,
            norm_len,
        ];

        let response = self.net.feed_forward(inputs);
        let mut max_ind = 0;
        for i in 1..response.len() {
            if response[i] > response[max_ind] {
                max_ind = i;
            }
        }

        // Action space: 0: Straight, 1: Turn Left, 2: Turn Right
        self.dir = match max_ind {
            0 => self.dir,
            1 => self.dir.turn_left(),
            2 => self.dir.turn_right(),
            _ => self.dir,
        };

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
        if !(self.pos[0].0 <= 9 && self.pos[0].0 >= 0 && self.pos[0].1 <= 9 && self.pos[0].1 >= 0) {
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

        // Proximity score shaping
        if self.is_alive {
            let new_dist = (self.pos[0].0 - apple_pos.0).abs() + (self.pos[0].1 - apple_pos.1).abs();
            if new_dist < old_dist {
                self.proximity_score += 1.0;
            } else if new_dist > old_dist {
                self.proximity_score -= 1.5;
            }
        }

        // Lifetime update
        self.lifetime += 1;
    }

    pub fn on_death(&mut self) {
        self.score = (self.n_apples as f32 * 1000.0 + self.proximity_score).max(0.1);
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

        // Reserve 1 slot for a random immigrant if population allows
        let n_offspring = if n_snakes > 2 {
            n_snakes - 2
        } else {
            0
        };

        for _ in 0..n_offspring {
            let p1_idx = dist.sample(&mut self.rng);
            let p2_idx = dist.sample(&mut self.rng);
            let child = GenAlgo::cross(&self.pops[p1_idx], &self.pops[p2_idx], &mut self.rng);
            new_gen.push(child);
        }

        // Mutate offspring only (preserving elite at index 0)
        if new_gen.len() > 1 {
            GenAlgo::mutate(&mut new_gen[1..], &mut self.rng);
        }

        // Introduce 1 random immigrant
        if new_gen.len() < n_snakes {
            new_gen.push(Snake::new(&mut self.rng));
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
        let mut_chance = 0.08;

        for snake in snakes.iter_mut() {
            let mut strand = snake.net.serialize();
            for val in strand.iter_mut() {
                if rng.gen_bool(mut_chance) {
                    *val += rng.gen_range(-0.25..=0.25);
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
    fn test_center_spawning() {
        let mut rng = rand::thread_rng();
        let s = Snake::new(&mut rng);
        assert_eq!(s.pos, vec![(4, 5), (4, 6), (4, 7)]);
        assert_eq!(s.dir, Direction::Up);
        assert!(s.is_alive);
        assert_eq!(s.n_apples, 0);
        assert_eq!(s.steps_left, 100 + 3 * 2);
    }

    #[test]
    fn test_direction_turns() {
        assert_eq!(Direction::Up.turn_left(), Direction::Left);
        assert_eq!(Direction::Up.turn_right(), Direction::Right);

        assert_eq!(Direction::Down.turn_left(), Direction::Right);
        assert_eq!(Direction::Down.turn_right(), Direction::Left);

        assert_eq!(Direction::Left.turn_left(), Direction::Down);
        assert_eq!(Direction::Left.turn_right(), Direction::Up);

        assert_eq!(Direction::Right.turn_left(), Direction::Up);
        assert_eq!(Direction::Right.turn_right(), Direction::Down);

        let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
        for d in dirs {
            // 4 left turns or 4 right turns cycle back to original direction
            assert_eq!(d.turn_left().turn_left().turn_left().turn_left(), d);
            assert_eq!(d.turn_right().turn_right().turn_right().turn_right(), d);

            // Turn left and turn right are inverse operations
            assert_eq!(d.turn_left().turn_right(), d);
            assert_eq!(d.turn_right().turn_left(), d);
        }
    }

    #[test]
    fn test_egocentric_actions_prevent_180() {
        let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
        let opposite = |d: Direction| match d {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        };

        for d in dirs {
            let opp = opposite(d);
            // Straight action
            assert_ne!(d, opp);
            // Turn left action
            assert_ne!(d.turn_left(), opp);
            // Turn right action
            assert_ne!(d.turn_right(), opp);
        }
    }

    fn make_deterministic_snake(rng: &mut ThreadRng, action: usize) -> Snake {
        let mut s = Snake::new(rng);
        let len = s.net.serialize().len();
        let mut strand = vec![0.0; len];
        // Output layer has 3 neurons, each with 8 weights + 1 bias = 9 values (27 total).
        // Setting the chosen action's bias to 1.0 ensures feed_forward() deterministically outputs action.
        let bias_idx = (len - 27) + action * 9 + 8;
        strand[bias_idx] = 1.0;
        s.net.deserialize(strand);
        s
    }

    #[test]
    fn test_self_collision_termination() {
        let mut rng = rand::thread_rng();
        // Snake configured to turn left (action 1)
        let mut s = make_deterministic_snake(&mut rng, 1);
        // Setup snake coiled so that turning left immediately hits its own body:
        // Head at (2, 2) moving Right. Body coiled around.
        // Turning left changes direction to Up, moving head into (2, 1) where body cell is.
        s.pos = vec![(2, 2), (1, 2), (1, 1), (2, 1), (3, 1)];
        s.dir = Direction::Right;

        s.update((9, 9));
        assert!(!s.is_alive, "Self-collision must kill snake via update()");
    }

    #[test]
    fn test_wall_collision_termination() {
        let mut rng = rand::thread_rng();
        // Snake configured to move straight (action 0)
        let mut s = make_deterministic_snake(&mut rng, 0);
        // Snake at top-left edge moving Left into the wall
        s.pos = vec![(0, 0), (0, 1), (0, 2)];
        s.dir = Direction::Left;

        s.update((5, 5));
        assert!(!s.is_alive, "Wall collision must kill snake via update()");
    }

    #[test]
    fn test_tail_cell_not_treated_as_obstacle() {
        let mut rng = rand::thread_rng();
        let mut s = make_deterministic_snake(&mut rng, 0); // Straight
        // Tail is directly ahead of head:
        // Head at (2, 2) moving Right. Tail at (3, 2).
        s.pos = vec![(2, 2), (2, 3), (3, 3), (3, 2)];
        s.dir = Direction::Right;

        // Moving straight ahead into the cell vacated by tail
        s.update((5, 5));
        assert!(s.is_alive, "Snake should safely enter cell vacated by its tail");
        assert_eq!(s.pos[0], (3, 2));
    }

    #[test]
    fn test_starvation_termination() {
        let mut rng = rand::thread_rng();
        let mut s = Snake::new(&mut rng);
        s.steps_left = 1;
        s.update((4, 0));
        assert!(s.is_alive);
        assert_eq!(s.steps_left, 0);

        // Next update starves the snake
        s.update((4, 0));
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
        s.proximity_score = 15.0;
        s.on_death();
        assert!((s.score - (4.0 * 1000.0 + 15.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_looper_zero_apples_penalized() {
        let mut rng = rand::thread_rng();
        let mut s = Snake::new(&mut rng);
        s.n_apples = 0;
        s.lifetime = 100;
        s.proximity_score = -25.0;
        s.on_death();
        assert!((s.score - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_crash_and_starvation_scoring_parity() {
        let mut rng = rand::thread_rng();
        let mut crash_snake = Snake::new(&mut rng);
        crash_snake.n_apples = 2;
        crash_snake.proximity_score = 10.0;
        crash_snake.steps_left = 50; // crashed with steps remaining
        crash_snake.on_death();

        let mut starve_snake = Snake::new(&mut rng);
        starve_snake.n_apples = 2;
        starve_snake.proximity_score = 10.0;
        starve_snake.steps_left = 0; // starved
        starve_snake.on_death();

        assert!((crash_snake.score - 2010.0).abs() < f32::EPSILON);
        assert!((starve_snake.score - 2010.0).abs() < f32::EPSILON);
        assert!((crash_snake.score - starve_snake.score).abs() < f32::EPSILON);
    }

    #[test]
    fn test_explorer_outscores_2x2_looper() {
        let mut rng = rand::thread_rng();
        let apple = (9, 9);

        // 2x2 Looper: endlessly turns right in a 2x2 loop until starvation
        let mut looper = make_deterministic_snake(&mut rng, 2);
        while looper.is_alive {
            looper.update(apple);
        }
        looper.on_death();

        // Explorer: approaches apple directly
        let mut explorer = make_deterministic_snake(&mut rng, 0);
        let explorer_apple = (4, 0);
        for _ in 0..5 {
            explorer.update(explorer_apple);
        }
        explorer.on_death();

        assert!((looper.score - 0.1).abs() < f32::EPSILON);
        assert!(
            explorer.score > looper.score,
            "Explorer score ({}) must be strictly higher than looper score ({})",
            explorer.score,
            looper.score
        );
    }

    #[test]
    fn test_proximity_score_tracking() {
        let mut rng = rand::thread_rng();
        // Snake configured to move straight (action 0)
        let mut s = make_deterministic_snake(&mut rng, 0);
        // Default spawn: head at (4, 5), dir Up.
        // Step 1: Apple at (4, 0). Snake moves straight Up to (4, 4).
        // Distance decreases from 5 to 4 -> proximity reward +1.0.
        s.update((4, 0));
        assert!(s.is_alive);
        assert_eq!(s.pos[0], (4, 4));
        assert!((s.proximity_score - 1.0).abs() < f32::EPSILON);

        // Step 2: Apple placed behind at (4, 9). Snake moves straight Up to (4, 3).
        // Distance increases from 5 to 6 -> proximity penalty -1.5.
        s.update((4, 9));
        assert!(s.is_alive);
        assert_eq!(s.pos[0], (4, 3));
        assert!((s.proximity_score - (-0.5)).abs() < f32::EPSILON);
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

    #[test]
    fn test_evolve_population_size_and_immigrant() {
        let rng = rand::thread_rng();
        let mut algo = GenAlgo::new(rng, 10);
        algo.evolve();
        assert_eq!(algo.pops.len(), 10);
    }
}

