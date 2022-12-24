use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashSet};
use std::{
    error::Error,
    fmt::Debug,
    io::{stdin, BufRead},
};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
enum Blizzard {
    N,
    E,
    S,
    W,
}

impl Blizzard {
    fn next(&self, x: usize, y: usize, width: usize, height: usize) -> (usize, usize) {
        let (x1, y1) = match self {
            Blizzard::N => (0, -1),
            Blizzard::E => (1, 0),
            Blizzard::S => (0, 1),
            Blizzard::W => (-1, 0),
        };

        let (x1, y1) = (x as i32 + x1, y as i32 + y1);
        let x1 = if x1 < 0 {
            x1 + width as i32
        } else {
            x1 % width as i32
        } as usize;
        let y1 = if y1 == 0 {
            height as i32 - 1
        } else if y1 == height as i32 {
            1
        } else {
            y1
        } as usize;
        (x1, y1)
    }
}

trait BlizzardMap {
    fn next(&self, width: usize, height: usize) -> Self;
}

impl BlizzardMap for HashSet<((usize, usize), Blizzard)> {
    fn next(&self, width: usize, height: usize) -> Self {
        self.iter()
            .copied()
            .map(|((x, y), blizzard)| (blizzard.next(x, y, width, height), blizzard))
            .collect()
    }
}

impl TryFrom<u8> for Blizzard {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'^' => Self::N,
            b'>' => Self::E,
            b'<' => Self::W,
            b'v' => Self::S,
            _ => return Err("Invalid character"),
        })
    }
}

#[derive(Debug)]
enum Tile {
    Wall,
    Empty,
    Blizzard(Blizzard),
}

impl TryFrom<u8> for Tile {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'#' => Self::Wall,
            b'.' => Self::Empty,
            _ => Tile::Blizzard(Blizzard::try_from(value)?),
        })
    }
}

#[derive(Debug, Copy, Clone)]
struct State {
    x: usize,
    y: usize,
    round: usize,
}

impl Eq for State {}

impl PartialEq<Self> for State {
    fn eq(&self, other: &Self) -> bool {
        (self.x, self.y, self.round) == (other.x, other.y, other.round)
    }
}

impl PartialOrd<Self> for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.x + self.y, Reverse(self.round), self.x, self.y).cmp(&(
            other.x + other.y,
            Reverse(other.round),
            other.x,
            other.y,
        ))
    }
}

impl State {
    fn next(mut self, width: usize, height: usize) -> impl Iterator<Item = Self> {
        self.round += 1;

        [(0, 0), (0, 1), (0, -1), (1, 0), (-1, 0)]
            .into_iter()
            .filter_map(move |(dx, dy)| {
                let mut next = self;
                if next.x == 0 && next.y == 0 && dx == 0 && dy == 0 {
                    return Some(next);
                }
                next.x = (next.x as i32 + dx).try_into().ok()?;
                next.y = (next.y as i32 + dy).try_into().ok()?;
                if next.y == height && next.x == width - 1 {
                    return Some(next);
                }
                ((0..width).contains(&next.x) && (1..height).contains(&next.y)).then_some(next)
            })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let (width, height, blizzard) = {
        let lines = stdin().lock().lines();
        lines
            .flatten()
            .enumerate()
            .try_fold(
                (0, 0, HashSet::new()),
                |(mut max_x, mut max_y, mut blizzard), (y, line)| {
                    for (x, c) in line.bytes().enumerate() {
                        match Tile::try_from(c)? {
                            Tile::Wall => {
                                if x > 0 {
                                    max_x = max_x.max(x - 1);
                                }
                                if y > 0 {
                                    max_y = max_y.max(y);
                                }
                            }
                            Tile::Empty => (),
                            Tile::Blizzard(b) => {
                                blizzard.insert(((x - 1, y), b));
                            }
                        }
                    }
                    Ok::<_, Box<dyn Error>>((max_x, max_y, blizzard))
                },
            )
            .map_err(Box::<dyn Error>::from)?
    };
    let mut configurations = Vec::new();
    let mut current = HashSet::from_iter(blizzard);
    loop {
        if !configurations.contains(&current) {
            configurations.push(current.clone());
            current = current.next(width, height);
        } else {
            break;
        }
    }

    let mut queue = BinaryHeap::from([State {
        round: 0,
        x: 0,
        y: 0,
    }]);
    let mut seen = HashSet::new();
    let mut best = usize::MAX;
    while let Some(current) = queue.pop() {
        let distance = (width - 1 - current.x) + height - current.y;
        if current.round + distance > best {
            continue;
        }
        if current.x == width - 1 && current.y == height {
            best = best.min(current.round);
        } else {
            for state in current.next(width, height) {
                let key = (state.x, state.y, state.round % configurations.len());
                if seen.contains(&key) {
                    continue;
                }
                if !configurations[state.round % configurations.len()]
                    .iter()
                    .any(|&((x, y), _)| x == state.x && y == state.y)
                {
                    seen.insert(key);
                    queue.push(state);
                }
            }
        }
    }

    println!("{}", best);
    Ok(())
}

#[allow(dead_code)]
fn print(map: &HashSet<((usize, usize), Blizzard)>, width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            let c = if let Some(&((_, _), tile)) =
                map.iter().find(|((dx, dy), _)| x == *dx && y == *dy)
            {
                match tile {
                    Blizzard::N => "^",
                    Blizzard::E => ">",
                    Blizzard::S => "v",
                    Blizzard::W => "<",
                }
            } else {
                "."
            };
            print!("{}", c)
        }
        println!()
    }
}
