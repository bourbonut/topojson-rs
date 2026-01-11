use crate::topojson_structs::Transform;

pub struct ScaleUntransformer {
    x0: f64,
    y0: f64,
    kx: f64,
    ky: f64,
    dx: f64,
    dy: f64,
}

impl ScaleUntransformer {
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

impl ScaleUntransformer {
    pub fn call(&mut self, input: &[f64; 2], i: usize) -> [f64; 2] {
        if i == 0 {
            self.x0 = 0.;
            self.y0 = 0.;
        }
        let x1 = ((input.get(0).unwrap_or(&f64::NAN) - self.dx) / self.kx + 0.5).floor();
        let y1 = ((input.get(1).unwrap_or(&f64::NAN) - self.dy) / self.ky + 0.5).floor();
        let output = [x1 - self.x0, y1 - self.y0];
        self.x0 = x1;
        self.y0 = y1;
        output
    }
}
