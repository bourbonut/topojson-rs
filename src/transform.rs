use crate::topojsons::Transform;

pub trait Transformer {
    fn call(&mut self, input: &[f64; 2], i: usize) -> [f64; 2];
}

pub struct IdentityTransformer;

impl IdentityTransformer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Transformer for IdentityTransformer {
    fn call(&mut self, input: &[f64; 2], _i: usize) -> [f64; 2] {
        *input
    }
}

pub struct ScaleTransformer {
    x0: f64,
    y0: f64,
    kx: f64,
    ky: f64,
    dx: f64,
    dy: f64,
}

impl ScaleTransformer {
    pub fn new(transform: &Transform) -> Self {
        Self {
            x0: 0.,
            y0: 0.,
            kx: transform.scale[0],
            ky: transform.scale[1],
            dx: transform.translate[0],
            dy: transform.translate[1],
        }
    }
}

impl Transformer for ScaleTransformer {
    fn call(&mut self, input: &[f64; 2], i: usize) -> [f64; 2] {
        if i == 0 {
            self.x0 = 0.;
            self.y0 = 0.;
        }
        let mut output: [f64; 2] = *input;
        self.x0 += input.first().unwrap_or(&f64::NAN);
        self.y0 += input.get(1).unwrap_or(&f64::NAN);
        output[0] = self.x0 * self.kx + self.dx;
        output[1] = self.y0 * self.ky + self.dy;
        output
    }
}
