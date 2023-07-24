use std::{collections::VecDeque, fmt::Display, fs::File, io::Read, path::Path, vec};

type TapeEntry = String;
static DEFAULT_ENTRY: &str = "0";

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
    state: String,
    entry: TapeEntry,
    new_state: Option<String>,
    new_entry: TapeEntry,
    direction: Direction,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.pad(&format!(
            "({}, {}) -> ({}, {}, {})",
            self.state,
            self.entry,
            match &self.new_state {
                Some(state) => state,
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
        if line.len() == 0 {
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
        let source_entry = line[1].to_string();
        let target_state = line[3].to_string();
        let target_entry = line[4].to_string();

        let direction = if line[5] == "L" {
            Direction::Left
        } else if line[5] == "R" {
            Direction::Right
        } else {
            panic!("couldn't parse direction '{}'", line[5])
        };

        let target_state = if target_state == "Halt" {
            None
        } else {
            Some(target_state)
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
    state: Option<String>,
    instructions: Vec<Instruction>,
    tape: VecDeque<TapeEntry>,
    pos: usize,
    offset: usize,

    num_steps: u128,
}

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

                return TuringMachine {
                    state: Some("A".to_string()),
                    instructions,
                    tape: vec![DEFAULT_ENTRY.to_string()].into(),
                    pos: 0,
                    offset: 0,
                    num_steps: 0,
                };
            }
        }
    }

    pub fn step(&mut self) -> bool {
        match &self.state {
            None => false,
            Some(state) => {
                self.num_steps += 1;
                let mut instruction = None;
                for inst in &self.instructions {
                    if state == &inst.state && self.tape[self.pos] == inst.entry {
                        instruction = Some(inst);
                    }
                }

                match instruction {
                    Some(instruction) => {
                        self.state = instruction.new_state.clone();
                        self.tape[self.pos] = instruction.new_entry.clone();

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
                        true
                    }
                    None => {
                        dbg!(self);
                        panic!("No Instruction matched Touringmachine");
                    }
                }
            }
        }
    }

    fn extend_left(&mut self) {
        self.tape.push_front(DEFAULT_ENTRY.to_string());
        self.pos += 1;
        self.offset += 1;
    }

    fn extend_right(&mut self) {
        self.tape.push_back(DEFAULT_ENTRY.to_string());
    }

    pub fn print_tape(&self, include_pos_marker: bool) {
        let mut tape = "".to_string();
        for entry in &self.tape {
            tape += &format!(" {entry}");
        }

        let mut instruction = None;
        match &self.state {
            Some(state) => {
                for inst in &self.instructions {
                    if state == &inst.state && self.tape[self.pos] == inst.entry {
                        instruction = Some(inst);
                    }
                }
            }
            None => {}
        }

        let state = match &self.state {
            Some(state) => state,
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
        for instruction in &self.instructions {
            println!("{instruction}");
        }
        println!("");
    }

    pub fn eval_busy_bever(&self) {
        let mut ones: i128 = 0;
        for entry in &self.tape {
            if entry == "1" {
                ones += 1;
            }
        }
        println!("Busy Bever: {} ones after {} steps", ones, self.num_steps);
    }
}
