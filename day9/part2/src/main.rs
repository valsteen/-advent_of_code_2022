use std::collections::HashSet;
use std::error::Error;
use std::io::{stdin, BufRead};

struct State<const LEN: usize> {
    visited: HashSet<(isize, isize)>,
    knots: [(isize, isize);LEN],
}

impl <const LEN: usize> Default for State<LEN> {
    fn default() -> Self {
        let visited = HashSet::from_iter([(0, 0)]);
        Self {
            visited,
            knots: [(0,0); LEN],
        }
    }
}

const LEN : usize = 10 ;

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let state = lines
        .flatten()
        .try_fold(State::<LEN>::default(), |mut state, line| {
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
                state.knots[0].0 += direction.0;
                state.knots[0].1 += direction.1;

                for i in 1..LEN {
                    let previous = state.knots[i-1];
                    let current = &mut state.knots[i];

                    let dx = previous.0 - current.0;
                    let dy = previous.1 - current.1;
                    if dx.abs() > 1 || dy.abs() > 1 {
                        current.0 += dx.abs().min(1) * dx.signum();
                        current.1 += dy.abs().min(1) * dy.signum();
                    }
                }
                state.visited.insert(*state.knots.last().unwrap());
            }

            Ok::<_, Box<dyn Error>>(state)
        })?;

    println!("{}", state.visited.len());
    Ok(())
}
