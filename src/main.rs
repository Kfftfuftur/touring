mod turing;
use std::{path::PathBuf, time::Instant};

use clap::{command, Parser};
use turing::TuringMachine;

#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    /// Filename of the Turing-Machine to load.
    filename: PathBuf,
}

fn main() {
    let args = Args::parse();
    let mut tm = TuringMachine::new(&args.filename);

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
