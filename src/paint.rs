use crate::shape::Shape;

pub fn draw_shape(canvas: &mut [u8], w: u32, h: u32, shape: &Shape, color: [u8; 4]) {
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
                canvas[i..i+4].copy_from_slice(&color);
            }
        }
    }
}

pub fn with_shape(src: &[u8], w: u32, h: u32, shape: &Shape, color: [u8; 4]) -> Vec<u8> {
    let mut dst = src.to_vec();
    draw_shape(&mut dst, w, h, shape, color);
    dst
}