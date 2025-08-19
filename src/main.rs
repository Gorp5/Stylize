mod config;
mod io;
mod shape;
mod paint;
mod gpu;
mod score;

use anyhow::Result;
use rand::SeedableRng;

fn main() -> Result<()> {
    let cfg = config::Config::parse();
    let (orig_data, w, h) = io::load_image(&cfg.input)?;

    let mut canvas = vec![0u8; (w * h * 4) as usize];
    let ctx = gpu::device::GpuContext::init()?;
    let scorer = score::GpuScorer::new(&ctx, w, h);

    let orig_texture = gpu::util::upload_image_to_texture(&ctx.device, &ctx.queue, &orig_data, w, h)?;

    let mut rng = match cfg.seed {
        Some(s) => rand::rngs::StdRng::seed_from_u64(s),
        None => rand::rngs::StdRng::from_entropy(),
    };

    for i in 0..cfg.iterations {
        let shape = shape::random_shape(w, h, &mut rng);
        let color = shape::avg_color_cpu(&shape, &orig_data, w, h);
        let trial = paint::with_shape(&canvas, w, h, &shape, color);

        let trial_texture = gpu::util::upload_image_to_texture(&ctx.device, &ctx.queue, &orig_data, w, h)?;
        let current_texture = gpu::util::upload_image_to_texture(&ctx.device, &ctx.queue, &orig_data, w, h)?;

        let before = scorer.score(&ctx, &orig_texture.view(), &current_texture.view(), w, h)?;
        let after  = scorer.score(&ctx, &orig_texture.view(), &trial_texture.view(), w, h)?;

        if after < before {
            canvas = trial;
        }

        if i % cfg.save_every == 0 {
            io::save_image(&format!("iter_{i:04}.png"), &canvas, w, h)?;
        }
    }

    io::save_image(&cfg.output, &canvas, w, h)?;
    Ok(())
}
