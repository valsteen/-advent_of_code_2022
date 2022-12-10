use recap::Recap;
use serde::Deserialize;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::mem;
use std::str::FromStr;

#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"^addx (?P<amount>-?[0-9]+)$"#)]
struct AddX {
    amount: isize,
}

#[derive(Debug)]
enum Instruction {
    Noop,
    AddX(AddX),
}

impl FromStr for Instruction {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self::AddX).or_else(|_| {
            Ok((s == "noop")
                .then_some(Self::Noop)
                .ok_or("invalid instruction")?)
        })
    }
}

impl Instruction {
    fn cycles(&self) -> usize {
        match self {
            Instruction::Noop => 1,
            Instruction::AddX(_) => 2,
        }
    }
}

struct Execution {
    instruction: Instruction,
    remaining: usize,
}

impl Execution {
    fn new(instruction: Instruction) -> Self {
        Self {
            remaining: instruction.cycles(),
            instruction,
        }
    }

    fn execute(mut self, state: isize) -> (Option<Self>, isize) {
        match &self.instruction {
            Instruction::Noop => (None, state),
            Instruction::AddX(add) => {
                self.remaining -= 1;
                if self.remaining == 0 {
                    (None, state + add.amount)
                } else {
                    (Some(self), state)
                }
            }
        }
    }
}

struct State<T: Iterator<Item = Result<Instruction, Box<dyn Error>>>> {
    current_line: String,
    instructions: T,
    current_execution: Option<Execution>,
    x: isize,
}

enum Iteration {
    Error(Box<dyn Error>),
    Continue,
    Display(String),
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines().flatten();

    let compute = (0..).scan(
        State {
            current_line: String::new(),
            instructions: lines.map(|line| line.parse::<Instruction>()),
            current_execution: None,
            x: 0,
        },
        |state, _| {
            state.current_line +=
                if (state.x..=state.x + 2).contains(&(state.current_line.len() as isize)) {
                    "#"
                } else {
                    " "
                };

            let (execution, next) = if let Some(execution) = state.current_execution.take() {
                execution
            } else {
                match state.instructions.next() {
                    Some(Ok(instruction)) => Execution::new(instruction),
                    Some(Err(e)) => return Some(Iteration::Error(e)),
                    None => return None,
                }
            }
            .execute(state.x);
            state.current_execution = execution;
            state.x = next;

            if state.current_line.len() == 40 {
                Some(Iteration::Display(mem::take(&mut state.current_line)))
            } else {
                Some(Iteration::Continue)
            }
        },
    );

    for iteration in compute {
        match iteration {
            Iteration::Error(err) => Err(err)?,
            Iteration::Continue => (),
            Iteration::Display(line) => {
                println!("{}", line);
            }
        }
    }
    Ok(())
}
