use std::collections::HashSet;
use std::error::Error;
use std::io::{stdin, BufRead};

struct State {
    visited: HashSet<(isize, isize)>,
    head: (isize, isize),
    tail: (isize, isize),
}

impl Default for State {
    fn default() -> Self {
        let visited = HashSet::from_iter([(0, 0)]);
        Self {
            visited,
            head: (0, 0),
            tail: (0, 0),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let state = lines
        .flatten()
        .try_fold(State::default(), |mut state, line| {
            let bytes = line.as_bytes();
            bytes.get(1).filter(|c| **c == b' ').ok_or("invalid line")?;
            let steps: usize = line.get(2..).ok_or("invalid line")?.parse()?;
            let direction = match bytes[0] {
                b'L' => (-1, 0),
                b'R' => (1, 0),
                b'U' => (0, -1),
                b'D' => (0, 1),
                _ => Err("invalid line")?,
            };
            for _ in 0..steps {
                state.head.0 += direction.0;
                state.head.1 += direction.1;
                let dx = state.head.0 - state.tail.0;
                let dy = state.head.1 - state.tail.1;
                if dx.abs() > 1 || dy.abs() > 1 {
                    state.tail.0 += dx.abs().min(1) * dx.signum();
                    state.tail.1 += dy.abs().min(1) * dy.signum();
                }
                state.visited.insert((state.tail.0, state.tail.1));
            }
            Ok::<_, Box<dyn Error>>(state)
        })?;

    println!("{}", state.visited.len());
    Ok(())
}
