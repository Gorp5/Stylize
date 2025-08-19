use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

#[derive(Debug, Clone)]
pub enum Shape {
    Circle { x: f32, y: f32, r: f32 },
    Rect { x: u32, y: u32, w: u32, h: u32 },
}

pub fn random_shape(w: u32, h: u32, rng: &mut StdRng) -> Shape {
    if rng.gen_bool(0.5) {
        let r = rng.gen_range(5.0..(w.min(h) as f32) / 4.0);
        let x = rng.gen_range(r..(w as f32 - r));
        let y = rng.gen_range(r..(h as f32 - r));
        Shape::Circle { x, y, r }
    } else {
        let rw = rng.gen_range(1..=w / 4);
        let rh = rng.gen_range(1..=h / 4);
        let x = rng.gen_range(0..w - rw);
        let y = rng.gen_range(0..h - rh);
        Shape::Rect { x, y, w: rw, h: rh }
    }
}

/// Compute average RGB of pixels covered by the shape.
pub fn avg_color_cpu(shape: &Shape, canvas: &[u8], w: u32, h: u32) -> [u8; 4] {
    let (mut sr, mut sg, mut sb, mut count) = (0u64, 0u64, 0u64, 0u64);
    for yy in 0..h {
        for xx in 0..w {
            let inside = match shape {
                Shape::Circle { x, y, r } => {
                    let dx = xx as f32 - *x;
                    let dy = yy as f32 - *y;
                    dx*dx + dy*dy <= *r * *r
                }
                Shape::Rect { x, y, w: rw, h: rh } => {
                    xx >= *x && xx < *x + *rw && yy >= *y && yy < *y + *rh
                }
            };
            if inside {
                let i = ((yy * w + xx) * 4) as usize;
                sr += canvas[i] as u64;
                sg += canvas[i + 1] as u64;
                sb += canvas[i + 2] as u64;
                count += 1;
            }
        }
    }
    if count == 0 {
        [0, 0, 0, 255]
    } else {
        [
            (sr / count) as u8,
            (sg / count) as u8,
            (sb / count) as u8,
            255,
        ]
    }
}