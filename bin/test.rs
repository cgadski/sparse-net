use sparse_net::*;

const LAYERS: usize = 3;
const PER_LAYER: usize = 20;

#[derive(Copy, Clone, Debug)]
enum Neuron {
    Input,
    Hidden { layer: usize, index: usize },
    Output 
}

impl Into<usize> for Neuron {
    fn into(self) -> usize {
        match self {
            Neuron::Input => 0,
            Neuron::Output => LAYERS * PER_LAYER + 1,
            Neuron::Hidden { layer, index } => layer * PER_LAYER + index + 1
        }
    } 
}

impl Neuron {
    fn enumerate() -> impl Iterator<Item=Neuron> {
        (0..LAYERS).flat_map(|layer| {
            (0..PER_LAYER).map(move |index| {
                Neuron::Hidden { layer, index }
            })
        }).chain(vec![Neuron::Input, Neuron::Output].into_iter())
    }

    fn from_int(i: usize) -> Neuron {
        if i == 0 {
            Neuron::Input
        } else if i == LAYERS * PER_LAYER + 1 {
            Neuron::Output
        } else {
            Neuron::Hidden {
                layer: (i - 1) / PER_LAYER,
                index: (i - 1) % PER_LAYER
            }
        }
    }
}

fn main() {
    let neurons: usize = 5;

    let mut coo = CooMatrix::new();
    for neuron in Neuron::enumerate() {
        match neuron {
            Neuron::Input => {
                for i in 0..PER_LAYER {
                    coo.push(Neuron::Hidden { layer: 0, index: i }, neuron, 1.);
                }
            },
            Neuron::Hidden { layer, index } => {
                if index == LAYERS - 1 {
                    coo.push(Neuron::Output, Neuron::Hidden { layer, index: 0 }.into(), 1.);
                } else {
                    coo.push(Neuron::Hidden { layer: layer + 1, index }, neuron, 1.);
                    coo.push(Neuron::Hidden { layer: layer + 1, index: (index + 1) % PER_LAYER }, neuron, 1.);
                }
            },
            _ => {}
        }
    }

    // println!("{:?}", coo);

    let net: SparseNet = SparseNet::from_coo(
        vec![0.; neurons], 
        &coo, 
        1, 
        1
    );

    net.to_graphviz().unwrap();

    // let mut data = net.new_data();
    // net.load_input(&mut data, &[1.]);
    // net.forward(&mut data);
    // let output = net.get_output(&data);
    // println!("{:?}", &data.activations);
}