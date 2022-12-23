use std::collections::{HashMap, HashSet};
use std::{
    error::Error,
    fmt::Debug,
    io::{stdin, BufRead},
};

#[derive(Debug, Copy, Clone)]
enum DestinationState {
    Free,
    Taken(i32, i32),
    Blocked,
}

impl Default for DestinationState {
    fn default() -> Self {
        Self::Free
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum Proposition {
    N,
    S,
    W,
    E,
}

impl Proposition {
    fn neighbours(self) -> [(i32, i32); 3] {
        match self {
            Proposition::N => [(-1, -1), (0, -1), (1, -1)],
            Proposition::S => [(-1, 1), (0, 1), (1, 1)],
            Proposition::W => [(-1, -1), (-1, 0), (-1, 1)],
            Proposition::E => [(1, -1), (1, 0), (1, 1)],
        }
    }

    fn step(self) -> (i32, i32) {
        match self {
            Proposition::N => (0, -1),
            Proposition::S => (0, 1),
            Proposition::W => (-1, 0),
            Proposition::E => (1, 0),
        }
    }
}

const PROPOSITIONS: [Proposition; 4] = [
    Proposition::N,
    Proposition::S,
    Proposition::W,
    Proposition::E,
];

fn next_propositions(round: usize) -> impl Iterator<Item = Proposition> {
    (0..=3).map(move |i| PROPOSITIONS[(round - 1 + i) % 4])
}

fn map_bounds(map: &HashSet<(i32, i32)>) -> (i32, i32, i32, i32) {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (0, 0, i32::MIN, i32::MIN);
    for &(x, y) in map {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    (min_x, min_y, max_x, max_y)
}

#[allow(dead_code)]
fn print(map: &HashSet<(i32, i32)>) {
    let (min_x, min_y, max_x, max_y) = map_bounds(map);
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if map.contains(&(x, y)) {
                print!("#")
            } else {
                print!(".")
            }
        }
        println!()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut map = {
        let lines = stdin().lock().lines();
        lines
            .flatten()
            .enumerate()
            .try_fold(HashSet::new(), |mut acc, (y, line)| {
                for (x, c) in line.bytes().enumerate() {
                    match c {
                        b'.' => (),
                        b'#' => {
                            acc.insert((x as i32, y as i32));
                        }
                        _ => return Err::<_, Box<dyn Error>>("invalid character".into()),
                    }
                }
                Ok(acc)
            })?
    };

    let mut propositions = HashMap::new();
    let mut elf_buffer = HashSet::new();

    let mut round = 0;
    let result = loop {
        round += 1;
        elf_buffer.clear();
        elf_buffer.extend(map.iter().map(|&(x, y)| (x, y)));

        'elf: for (x, y) in elf_buffer.drain() {
            if [
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
            ]
                .into_iter()
                .all(|(dx, dy)| !map.contains(&(x + dx, y + dy)))
            {
                continue 'elf;
            }

            let mut action = None;

            'proposition: for proposition in next_propositions(round) {
                for (x1, y1) in proposition.neighbours() {
                    if map.contains(&(x + x1, y + y1)) {
                        continue 'proposition;
                    }
                }
                action = Some(proposition);
                break 'proposition;
            }

            if let Some(action) = action {
                let (dx, dy) = action.step();
                let dx = x + dx;
                let dy = y + dy;

                let entry = propositions.entry((dx, dy)).or_default();

                *entry = match entry {
                    DestinationState::Free => DestinationState::Taken(x, y),
                    DestinationState::Blocked | DestinationState::Taken(_, _) => {
                        DestinationState::Blocked
                    }
                };
            }
        }

        if propositions.is_empty() {
            break round;
        }

        elf_buffer.extend(map.drain());

        for ((origin_x, origin_y), (dx, dy)) in
        propositions.drain().filter_map(|((dx, dy), state)| {
            if let DestinationState::Taken(x, y) = state {
                Some(((x, y), (dx, dy)))
            } else {
                None
            }
        })
        {
            elf_buffer.remove(&(origin_x, origin_y));
            map.insert((dx, dy));
        }

        map.extend(elf_buffer.drain());
    };

    println!("{}", result);
    Ok(())
}
