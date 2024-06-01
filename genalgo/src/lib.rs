use rand::{
    distributions::{
        Distribution, WeightedIndex
    }, 
    rngs::ThreadRng, Rng
};
use nnet::*;

#[derive(Clone)]
pub struct Snake {
    net: NNet,
    pos: (u8, u8),
    score: f32,
}

pub struct GenAlgo {
    pops: Vec<Snake>,
    rng: ThreadRng
}

impl Snake {
    fn new(rng: &mut ThreadRng) -> Self {
        // May change the bprint later
        let net = NNet::new(&[8, 8, 8, 4], rng);
                
        let x: u8 = rng.gen_range(0..=9);
        let y: u8 = rng.gen_range(0..=9);

        Snake { net, pos: (x, y), score: 0.0 }
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
            .map(|s| s.score)
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
        let mut_chance = 0.3;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foo() {
        todo!()
    }
}
