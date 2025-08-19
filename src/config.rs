use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "Stylize")]
pub struct Config {
    /// Input image path
    #[arg(short, long)]
    pub input: String,

    /// Output image path
    #[arg(short, long, default_value = "output.png")]
    pub output: String,

    /// Number of iterations
    #[arg(short = 'n', long, default_value_t = 1000)]
    pub iterations: u32,

    /// Save every N iterations
    #[arg(long, default_value_t = 100)]
    pub save_every: u32,

    /// RNG seed (optional)
    #[arg(long)]
    pub seed: Option<u64>,
}