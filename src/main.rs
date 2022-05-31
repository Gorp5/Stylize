mod libs;

use libs::run;

pub fn main() {
    pollster::block_on(run());
}

