use rand::{
    rngs::ThreadRng, 
    Rng
};

#[derive(Debug, Clone)]
pub struct NNet {
    layers: Vec<Layer>,
    bprint: Vec<usize>
}

#[derive(Debug, Clone)]
struct Layer {
    neurons: Vec<Neuron>
}

#[derive(Debug, Clone)]
struct Neuron {
    weights: Vec<f32>,
    bias: f32
}

impl NNet {
    // The first element is the input size
    pub fn new(bprint: &[usize], rng: &mut ThreadRng) -> Self {
        let mut layers = Vec::with_capacity(bprint.len());
        
        for i in 1..bprint.len() {
            layers.push(Layer::new(bprint[i], bprint[i-1], rng));
        }

        let bprint = bprint.to_vec();
        NNet { layers, bprint }
    }

    pub fn feed_forward(&self, inputs: Vec<f32>) -> Vec<f32> {
        let mut inp_cp = inputs.clone();

        for layer in &self.layers {
            inp_cp = layer.feed_forward(&inp_cp);
        }

        NNet::softmax(inp_cp)
    }

    pub fn serialize(&self) -> Vec<f32> {
        let mut strand = Vec::new();
        
        for layer in &self.layers {
            let temp = layer.serialize();

            for n in temp {
                strand.push(n);
            }
        }

        strand
    }

    pub fn deserialize(&mut self, strand: Vec<f32>) {
        let mut new_layers: Vec<Layer> = Vec::with_capacity(self.layers.len());
        let mut count = 0;

        for i in 1..self.bprint.len() {
            let mut neurons = Vec::with_capacity(self.bprint[i]);

            for _ in 0..self.bprint[i] {
                let mut temp = Vec::new();

                for _ in 0..self.bprint[i-1] {
                    temp.push(strand[count]);    
                    count += 1;
                }

                neurons.push(Neuron { weights: temp, bias: strand[count] });
                count += 1;
            }

            new_layers.push(Layer { neurons });
        }

        self.layers = new_layers;
    }

    fn softmax(vals: Vec<f32>) -> Vec<f32> {
        let max_val = vals.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        
        let exp_vals: Vec<f32> = vals.iter()
            .map(|x| (x-max_val).exp())
            .collect();

        let sum: f32 = exp_vals.iter().sum();

        exp_vals.iter()
            .map(|x| x / sum)
            .collect()
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

    fn serialize(&self) -> Vec<f32> {
        let mut strand = Vec::new();

        for neuron in &self.neurons {
            for w in &neuron.weights {
                strand.push(*w);
            }
            strand.push(neuron.bias);
        }

        strand
    }
}

impl Neuron {
    fn new(pre_size: usize, rng: &mut ThreadRng) -> Self {
        let bias = rng.gen_range(-5.0..5.0);

        let mut weights = Vec::with_capacity(pre_size);
        for _ in 0..pre_size {
            weights.push(rng.gen_range(-5.0..5.0));
        }

        Neuron { weights, bias }
    }

    fn feed_forward(&self, inputs: &[f32]) -> f32 {
        let mut result: f32 = 0.0;

        for i in 0..inputs.len() {
            result += inputs[i] * self.weights[i];
        }

        // RELU activation function
        f32::max(result + self.bias, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neuron_dot_product_and_relu() {
        let neuron = Neuron {
            weights: vec![0.5, -1.0, 2.0],
            bias: -0.5,
        };
        // 0.5 * 2.0 + (-1.0) * 1.0 + 2.0 * 3.0 + (-0.5) = 1.0 - 1.0 + 6.0 - 0.5 = 5.5
        let out = neuron.feed_forward(&[2.0, 1.0, 3.0]);
        assert!((out - 5.5).abs() < 1e-6);

        // Negative sum clamped by ReLU to 0.0
        let clamped = neuron.feed_forward(&[-10.0, 0.0, 0.0]);
        assert_eq!(clamped, 0.0);
    }

    #[test]
    fn test_serialize_deserialize_fidelity() {
        let mut rng = rand::thread_rng();
        let blueprint = [8, 16, 4];
        let net1 = NNet::new(&blueprint, &mut rng);
        let strand1 = net1.serialize();

        let mut net2 = NNet::new(&blueprint, &mut rng);
        net2.deserialize(strand1.clone());
        let strand2 = net2.serialize();

        assert_eq!(strand1, strand2);

        let test_input = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let out1 = net1.feed_forward(test_input.clone());
        let out2 = net2.feed_forward(test_input);
        for (a, b) in out1.iter().zip(out2.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn test_softmax_properties() {
        let out = NNet::softmax(vec![1.0, 2.0, 3.0]);
        let sum: f32 = out.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
        assert!(out[2] > out[1]);
        assert!(out[1] > out[0]);
    }
}


