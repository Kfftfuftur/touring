use std::{collections::VecDeque, fmt::Display, fs::File, io::Read, path::Path, sync::RwLock, vec};

type TapeEntry = u8;
static DEFAULT_ENTRY: TapeEntry = 0;
static STATES_LOCK: RwLock<Vec<String>> = RwLock::new(vec![]);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(match self {
            Direction::Left => "Left",
            Direction::Right => "Right",
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Instruction {
    state: usize,
    entry: TapeEntry,
    new_state: Option<usize>,
    new_entry: TapeEntry,
    direction: Direction,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let states = STATES_LOCK.read().unwrap();
        f.pad(&format!(
            "({}, {}) -> ({}, {}, {})",
            states[self.state],
            self.entry,
            match self.new_state {
                Some(state) => &states[state],
                None => "Halt",
            },
            self.new_entry,
            self.direction
        ))
    }
}

enum InstructionParseError {
    EmptyLine,
    ParseError { why: String },
}

impl TryFrom<&str> for Instruction {
    type Error = InstructionParseError;

    fn try_from(line: &str) -> Result<Self, Self::Error> {
        let mut states = STATES_LOCK.write().unwrap();
        if line.is_empty() {
            return Err(InstructionParseError::EmptyLine);
        }

        let line: Vec<&str> = line.split_whitespace().collect();

        if line.len() != 6 {
            return Err(InstructionParseError::ParseError {
                why: format!(
                    "Invalid number of elements (found {}, expected 6)",
                    line.len()
                ),
            });
        }

        let source_state = line[0].to_string();
        let source_state = match states.iter().position(|state| state == &source_state) {
            Some(source_state) => source_state,
            None => {
                states.push(source_state);
                states.len() - 1
            }
        };

        let target_state = line[3].to_string();
        let target_state = if target_state == "Halt" {
            None
        } else {
            match states.iter().position(|state| state == &target_state) {
                Some(target_state) => Some(target_state),
                None => {
                    states.push(target_state);
                    Some(states.len() - 1)
                }
            }
        };

        let source_entry = match line[1].to_string().parse() {
            Ok(source_entry) => source_entry,
            Err(why) => {
                return Err(InstructionParseError::ParseError {
                    why: format!("unable to parse source entry: {why}"),
                })
            }
        };

        let target_entry = match line[4].to_string().parse() {
            Ok(target_entry) => target_entry,
            Err(why) => {
                return Err(InstructionParseError::ParseError {
                    why: format!("unable to parse target entry: {why}"),
                })
            }
        };

        let direction = if line[5] == "L" {
            Direction::Left
        } else if line[5] == "R" {
            Direction::Right
        } else {
            panic!("couldn't parse direction '{}'", line[5])
        };

        Ok(Instruction {
            state: source_state,
            entry: source_entry,
            new_state: target_state,
            new_entry: target_entry,
            direction,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TuringMachine {
    state: Option<usize>,
    instructions: Box<[Instruction]>,
    tape: VecDeque<TapeEntry>,
    pos: usize,
    offset: usize,

    pub num_steps: u128,
}

#[allow(dead_code)]
impl TuringMachine {
    pub fn new(path: &Path) -> Self {
        let mut instructions = vec![];

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        };

        let mut content = String::new();
        match file.read_to_string(&mut content) {
            Err(why) => panic!("Couldn't read {}: {}", path.display(), why),
            Ok(_size) => {
                for line in content.lines() {
                    match Instruction::try_from(line) {
                        Ok(instruction) => instructions.push(instruction),
                        Err(InstructionParseError::EmptyLine) => {}
                        Err(InstructionParseError::ParseError { why }) => {
                            panic!("Can't read instruction from line '{}': {}", &line, &why)
                        }
                    }
                }

                TuringMachine {
                    state: Some(0),
                    instructions: instructions.into(),
                    tape: vec![DEFAULT_ENTRY].into(),
                    pos: 0,
                    offset: 0,
                    num_steps: 0,
                }
            }
        }
    }

    pub fn step(&mut self) -> bool {
        match &self.state {
            None => false,
            Some(state) => {
                self.num_steps += 1;
                for instruction in self.instructions.iter() {
                    if state == &instruction.state && self.tape[self.pos] == instruction.entry {
                        self.state = instruction.new_state;
                        self.tape[self.pos] = instruction.new_entry;

                        match instruction.direction {
                            Direction::Left => {
                                if self.pos == 0 {
                                    self.extend_left();
                                }
                                self.pos -= 1;
                            }
                            Direction::Right => {
                                self.pos += 1;
                                if self.pos == self.tape.len() {
                                    self.extend_right();
                                }
                            }
                        }
                        return true;
                    }
                }
                let states = STATES_LOCK.read();
                match states {
                    Ok(states) => {
                        dbg!(&states);
                    }
                    Err(why) => {
                        println!("Can't get read-lock for states: {}", why);
                    }
                };
                dbg!(self);
                panic!("No Instruction matched Turing-Machine");
            }
        }
    }

    fn extend_left(&mut self) {
        self.tape.push_front(DEFAULT_ENTRY);
        self.pos += 1;
        self.offset += 1;
    }

    fn extend_right(&mut self) {
        self.tape.push_back(DEFAULT_ENTRY);
    }

    pub fn print_tape(&self, include_pos_marker: bool) {
        let states = STATES_LOCK.read().unwrap();
        let mut tape = "".to_string();
        for entry in &self.tape {
            tape += &format!(" {entry}");
        }

        let mut instruction = None;
        match &self.state {
            Some(state) => {
                for inst in self.instructions.iter() {
                    if state == &inst.state && self.tape[self.pos] == inst.entry {
                        instruction = Some(inst);
                    }
                }
            }
            None => {}
        }

        let state = match self.state {
            Some(state) => &states[state],
            None => "Halt",
        };

        let instruction = match instruction {
            Some(instruction) => format!("{}", instruction),
            None => "No Instruction".to_string(),
        };

        println!(
            "State: {}, {}, {} steps",
            state, &instruction, self.num_steps
        );
        println!("{}", tape);

        if include_pos_marker {
            let mut indicator = "".to_string();
            for i in 0..=self.tape.len() {
                let marker = if i == self.pos { "^" } else { " " };
                let frame = if i == self.offset || i == self.offset + 1 {
                    "|"
                } else {
                    " "
                };

                indicator = indicator + frame + marker;
            }
            println!("{}", indicator);
        }
    }

    pub fn print_instructions(&self) {
        println!("Instructions: ");
        for instruction in self.instructions.iter() {
            println!("{instruction}");
        }
        println!();
    }

    pub fn print_states(&self) {
        let states = STATES_LOCK.read().unwrap();
        println!("States: ");
        println!(" Number | Name ");
        println!("--------+------");
        for i in 0..states.len() {
            println!(" {:6} | '{}' ", i, { &states[i] })
        }
        println!();
    }

    pub fn eval_busy_bever(&self) -> (u128, u128, u128) {
        let mut ones: u128 = 0;
        let mut zeros: u128 = 0;

        for entry in &self.tape {
            if *entry == 1 {
                ones += 1;
            } else if *entry == 0 {
                zeros += 1;
            }
        }
        println!(
            "Busy Bever: {} ones, {} zeros, after {} steps",
            ones, zeros, self.num_steps
        );

        (ones, zeros, self.num_steps)
    }
}

#[test]
fn test_busy_bever_1() {
    let mut tm = TuringMachine::new(Path::new("examples/busy_bever/busy_bever_1.turing"));

    tm.print_states();
    tm.print_instructions();

    let mut num_steps = 0;
    while tm.step() {
        num_steps += 1;
    }

    let (ones, zeros, _steps) = tm.eval_busy_bever();

    assert_eq!(ones, 1);
    assert_eq!(zeros, 1);
    assert_eq!(num_steps, 1);
}

#[test]
fn test_busy_bever_2() {
    let mut tm = TuringMachine::new(Path::new("examples/busy_bever/busy_bever_2.turing"));

    tm.print_states();
    tm.print_instructions();

    let mut num_steps = 0;
    while tm.step() {
        num_steps += 1;
    }

    let (ones, zeros, _steps) = tm.eval_busy_bever();

    assert_eq!(ones, 4);
    assert_eq!(zeros, 0);
    assert_eq!(num_steps, 6);
}

#[test]
fn test_busy_bever_3() {
    let mut tm = TuringMachine::new(Path::new("examples/busy_bever/busy_bever_3.turing"));

    tm.print_states();
    tm.print_instructions();

    let mut num_steps = 0;
    while tm.step() {
        num_steps += 1;
    }

    let (ones, zeros, _steps) = tm.eval_busy_bever();

    assert_eq!(ones, 6);
    assert_eq!(zeros, 0);
    assert_eq!(num_steps, 14);
}

#[test]
fn test_busy_bever_4() {
    let mut tm = TuringMachine::new(Path::new("examples/busy_bever/busy_bever_4.turing"));

    tm.print_states();
    tm.print_instructions();

    let mut num_steps = 0;
    while tm.step() {
        num_steps += 1;
    }

    let (ones, zeros, _steps) = tm.eval_busy_bever();

    assert_eq!(ones, 13);
    assert_eq!(zeros, 1);
    assert_eq!(num_steps, 107);
}

#[test]
fn test_busy_bever_5() {
    let mut tm = TuringMachine::new(Path::new(
        "examples/busy_bever/busy_bever_5_best_currently_known.turing",
    ));

    tm.print_states();
    tm.print_instructions();

    let mut num_steps = 0;
    while tm.step() {
        num_steps += 1;
    }

    let (ones, zeros, _steps) = tm.eval_busy_bever();

    assert_eq!(ones, 4098);
    assert_eq!(zeros, 8191);
    assert_eq!(num_steps, 47176870);
}
