use rand::{rngs::ThreadRng, Rng};

pub struct NNet {
    layers: Vec<Layer>,
}

struct Layer {
    neurons: Vec<Neuron>
}

struct Neuron {
    weights: Vec<f32>,
    bias: f32
}

impl NNet {
    pub fn new(bprint: &[usize]) -> Self {
        let mut rng = rand::thread_rng();

        let mut layers = Vec::with_capacity(bprint.len());
        
        for i in 1..bprint.len() {
            layers.push(Layer::new(bprint[i], bprint[i-1], &mut rng));
        }

        NNet { layers }
    }

    // Returns a size 4 probability array
    pub fn feed_forward(&self, inputs: Vec<f32>) -> Vec<f32> {
        let mut inp_cp = inputs.clone();

        for layer in &self.layers {
            inp_cp = layer.feed_forward(&inp_cp);
        }

        inputs.to_vec()
    }
}

impl Layer {
    fn new(n_neurons: usize, n_pre: usize, rng: &mut ThreadRng) -> Self {
        let mut neurons = Vec::with_capacity(n_neurons);
        for _ in 0..n_neurons {
            neurons.push(Neuron::new(n_pre, rng));
        }

        Layer { neurons }
    }

    fn feed_forward(&self, inputs: &[f32]) -> Vec<f32> {
        let mut results = Vec::with_capacity(self.neurons.len());

        for i in 0..self.neurons.len() {
            results.push(self.neurons[i].feed_forward(&inputs));
        }

        results
    }
}

impl Neuron {
    fn new(pre_size: usize, rng: &mut ThreadRng) -> Self {
        let bias = rng.gen_range(-5.0..5.0);

        let mut weights = Vec::with_capacity(pre_size);
        for i in 0..pre_size {
            weights[i] = rng.gen_range(-5.0..5.0);
        }

        Neuron { weights, bias }
    }

    fn feed_forward(&self, inputs: &[f32]) -> f32 {
        let mut result: f32 = 0.0;

        for i in 0..inputs.len() {
            result += inputs[i] + self.weights[i];
        }

        result + self.bias
    }
}

mod tests {
    #[test]
    fn new_nn_test() {
        todo!()
    }
}
