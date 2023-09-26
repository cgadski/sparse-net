use rand::Rng;

use rand::rngs::ThreadRng;
use std::io::{self, Write};

type NUM = f32;

#[derive(Clone, Debug)]
pub struct CooEntry {
    i: usize,
    j: usize,
    value: NUM,
}

#[derive(Debug)]
pub struct CooMatrix(Vec<CooEntry>);

pub fn random_vector(slice: &mut [NUM]) {
    let mut rng: ThreadRng = rand::thread_rng();
    for i in 0..slice.len() {
        slice[i] = (2.0 * (rng.gen::<f32>() * 1000.0) / 1000.0) - 1.0;
    }
}

impl CooMatrix {
    pub fn new() -> CooMatrix {
        CooMatrix(Vec::new())
    }

    pub fn push<T: Into<usize>>(&mut self, i: T, j: T, value: NUM) {
        self.0.push(CooEntry { i: i.into(), j: j.into(), value })
    }

    pub fn random(&mut self, i: usize, j: usize, entries: usize) {
        let mut rng: ThreadRng = rand::thread_rng();
        self.0.clear();
        for _ in 0..entries {
            let entry = CooEntry {
                i: rng.gen_range(0..i),
                j: rng.gen_range(0..j),
                value: (2.0 * (rng.gen::<f32>() * 1000.0) / 1000.0) - 1.0,
            };
            self.0.push(entry);
        }
    }

    pub fn mult_into(&self, v: &Vec<NUM>, r: &mut Vec<NUM>) {
        for entry in &self.0 {
            r[entry.i] += entry.value * v[entry.j];
        }
    }

    pub fn show(&self) {
        for entry in &self.0 {
            println!("{:8} {:8} {}", entry.i, entry.j, entry.value);
        }
    }
}

#[derive(Clone, Debug)]
pub struct CsrMatrix {
    pub row_extents: Vec<usize>,
    pub col_indices: Vec<usize>,
    pub values: Vec<NUM>,
}

impl CsrMatrix {
    pub fn new() -> Self {
        CsrMatrix {
            row_extents: Vec::new(),
            col_indices: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn into_iter<'a>(&'a self) -> CsrIterator<'a> {
        CsrIterator {
            mat: &self,
            row: 0,
            offset: 0,
            col_idx: 0,
        }
    }

    pub fn to_graphviz(&self) -> io::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        writeln!(handle, "digraph {{")?;
        writeln!(handle, "  graph [rankdir=LR];")?;
        writeln!(handle, "  node [shape=point];")?;

        for entry in self.into_iter() {
            writeln!(handle, "  {} -> {};", entry.i, entry.j)?;
        }

        writeln!(handle, "}}")?;

        Ok(())
    }

    // requires sorting the entries of the matrix by row
    pub fn from_coo_transformed<F>(&mut self, coo: &CooMatrix, rows: usize, idx_map: F)
    where
        F: Fn(usize, usize) -> (usize, usize),
    {
        let mut indices: Vec<usize> = (0..coo.0.len()).collect();
        indices.sort_by(|x, y| {
            let x_entry: &CooEntry = &coo.0[*x];
            let y_entry: &CooEntry = &coo.0[*y];
            idx_map(x_entry.i, x_entry.j)
                .0
                .cmp(&idx_map(y_entry.i, y_entry.j).0)
        });

        {
            self.row_extents.clear();
            self.row_extents.reserve(rows);

            self.col_indices.clear();
            self.col_indices.reserve(coo.0.len());

            self.values.clear();
            self.values.reserve(coo.0.len());
        }

        let mut row = 0;
        let mut row_extent = 0;

        for entry_idx in indices {
            let entry: &CooEntry = &coo.0[entry_idx];
            let (i, j) = idx_map(entry.i, entry.j);

            while row < i {
                self.row_extents.push(row_extent);
                row_extent = 0;
                row += 1;
            }

            self.values.push(entry.value);
            self.col_indices.push(j);
            row_extent += 1;
        }

        self.row_extents.push(row_extent);
    }

    pub fn from_coo(&mut self, coo: &CooMatrix, rows: usize) {
        self.from_coo_transformed(&coo, rows, |i, j| (i, j))
    }

    pub fn from_coo_tr(&mut self, coo: &CooMatrix, cols: usize) {
        self.from_coo_transformed(&coo, cols, |i, j| (j, i))
    }
}

pub struct CsrIterator<'a> {
    mat: &'a CsrMatrix,
    row: usize,
    offset: usize,
    col_idx: usize,
}

impl<'a> Iterator for CsrIterator<'a> {
    type Item = CooEntry;
    fn next(&mut self) -> Option<Self::Item> {
        while (self.row < self.mat.row_extents.len()) && (self.mat.row_extents[self.row] < 1) {
            self.row += 1;
        }
        if self.row >= self.mat.row_extents.len() {
            return None;
        }
        let value_idx = self.offset + self.col_idx;
        let entry = CooEntry {
            i: self.row,
            j: self.mat.col_indices[value_idx],
            value: self.mat.values[value_idx],
        };
        self.col_idx += 1;
        if self.col_idx >= self.mat.row_extents[self.row] {
            self.col_idx = 0;
            self.offset += self.mat.row_extents[self.row];
            self.row += 1;
        }
        Some(entry)
    }
}

#[allow(dead_code)]
pub struct SparseNet {
    num_vertices: usize,
    num_edges: usize,

    pub biases: Vec<NUM>,
    edges_forward: CsrMatrix,
    edges_backward: CsrMatrix,

    pub input: usize,
    pub output: usize,
}

pub struct SparseNetData {
    pub activations: Vec<NUM>,

    pub d_biases: Vec<NUM>,
    pub d_weights: Vec<NUM>,
}

#[inline(always)]
pub fn relu(x: NUM) -> NUM {
    if x > 0.0 {
        x
    } else {
        0.0
    }
}

impl SparseNet {
    pub fn from_coo(biases: Vec<NUM>, edges: &CooMatrix, input: usize, output: usize) -> Self {
        let mut edges_forward = CsrMatrix::new();
        edges_forward.from_coo(edges, biases.len());

        let mut edges_backward = CsrMatrix::new();
        edges_backward.from_coo_tr(edges, biases.len());

        SparseNet {
            num_vertices: biases.len(),
            num_edges: edges.0.len(),
            biases,
            edges_forward,
            edges_backward,
            input,
            output,
        }
    }

    pub fn to_graphviz(&self) -> io::Result<()> {
        self.edges_forward.to_graphviz()
    }

    pub fn new_data(&self) -> SparseNetData {
        SparseNetData {
            activations: vec![0.; self.num_vertices],
            d_biases: vec![0.; self.num_vertices],
            d_weights: vec![0.; self.num_edges],
        }
    }

    pub fn load_input(&self, data: &mut SparseNetData, input: &[NUM]) {
        for i in 0..self.input {
            data.activations[i] = input[i];
        }
    }

    pub fn get_output<'a>(&self, data: &'a SparseNetData) -> &'a [NUM] {
        &data.activations[(data.activations.len() - self.output)..]
    }

    pub fn forward(&self, data: &mut SparseNetData) {
        let mut sum: NUM = 0.;
        let mut neuron = 0;
        for entry in self.edges_forward.into_iter() {
            while neuron < entry.i && neuron < self.num_vertices {
                if neuron >= self.input {
                    data.activations[neuron] = relu(sum + self.biases[neuron]);
                    sum = 0.;
                }
                neuron += 1;
            }
            sum += data.activations[entry.j] * entry.value;
        }
        data.activations[neuron] = relu(sum + self.biases[neuron]);
    }

    // pub fn backward(&self, data: &mut SparseNetData) {
    //     let mut sum: NUM = 0.;
    // }
}
