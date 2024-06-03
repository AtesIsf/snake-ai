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

        inp_cp
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

    pub fn deserialize(&self, strand: Vec<f32>) {
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
            result += inputs[i] + self.weights[i];
        }

        result + self.bias
    }
}

