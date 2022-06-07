mod compute_pipeline;
mod libs;
mod texture;

use libs::run;

pub fn main() {
    pollster::block_on(run());
}

/* Program Logic

Create Shapes

Determine How good they all are

 */
