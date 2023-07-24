mod turing;
use std::{path::Path, time::Instant, env::args};

use turing::TuringMachine;

fn main() {
    let args: Vec<_> = args().collect();
    let mut tm = TuringMachine::new(Path::new(&args[1]));

    tm.print_instructions();

    let start = Instant::now();
    //tm.print_tape(true);
    while tm.step() {
        //tm.print_tape(true);
    }

    println!("\nSimulation took {:?}", start.elapsed());
    tm.print_tape(false);
    tm.eval_busy_bever();
}
