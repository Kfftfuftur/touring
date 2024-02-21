mod turing;
use std::{env::args, path::Path, time::Instant};

use turing::TuringMachine;

fn main() {
    let args: Vec<_> = args().collect();
    let mut tm = TuringMachine::new(Path::new(&args[1]));

    tm.print_states();
    tm.print_instructions();

    let start = Instant::now();

    while tm.step() {}

    let elapsed = start.elapsed();

    let freq = (tm.num_steps as f32) / elapsed.as_secs_f32();

    println!("\nSimulation took {:.3?}", elapsed);
    println!("{:.3e} Iterations / second", freq);

    tm.eval_busy_bever();
}
