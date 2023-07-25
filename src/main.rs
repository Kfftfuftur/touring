mod turing;
use std::{path::Path, time::Instant, env::args};

use turing::TuringMachine;

fn main() {
    let args: Vec<_> = args().collect();
    let mut tm = TuringMachine::new(Path::new(&args[1]));

    tm.print_states();
    tm.print_instructions();

    let start = Instant::now();
    //tm.print_tape(true);
    while tm.step() {
        //tm.print_tape(true);
    }
    let elapsed = start.elapsed();

    let freq = (tm.num_steps as f32) / elapsed.as_secs_f32();

    println!("\nSimulation took {:.3?}", elapsed);
    
    if freq < 1e3 {
        println!("{:.3} Hz", freq);
    } else if freq < 1e6 {
        println!("{:.3} kHz", freq / 1e3);
    } else if freq < 1e9 {
        println!("{:.3} MHz", freq / 1e6);
    } else {
        println!("{:.3} GHz", freq / 1e9);
    }

    //tm.print_tape(false);
    tm.eval_busy_bever();
}
