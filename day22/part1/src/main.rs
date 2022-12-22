use std::ops::ControlFlow;
use std::{
    error::Error,
    fmt::Debug,
    io::{stdin, BufRead},
};

#[derive(Debug, Copy, Clone)]
enum Tile {
    Open,
    Wall,
    Wrap,
}

#[derive(Debug, Copy, Clone)]
enum Turn {
    Left,
    Right,
}

#[derive(Debug, Copy, Clone)]
enum Move {
    Forward(usize),
    Turn(Turn),
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum Facing {
    Right,
    Down,
    Left,
    Up,
}

impl Facing {
    fn apply(self, turn: Turn) -> Self {
        match (self, turn) {
            (Facing::Up, Turn::Left) => Self::Left,
            (Facing::Up, Turn::Right) => Self::Right,

            (Facing::Down, Turn::Left) => Self::Right,
            (Facing::Down, Turn::Right) => Self::Left,

            (Facing::Left, Turn::Left) => Self::Down,
            (Facing::Left, Turn::Right) => Self::Up,

            (Facing::Right, Turn::Left) => Self::Up,
            (Facing::Right, Turn::Right) => Self::Down,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct State {
    x: usize,
    y: usize,
    facing: Facing,
}

trait Map {
    fn at(&self, x: i32, y: i32) -> Tile;
}

impl Map for &[Vec<Tile>] {
    fn at(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || y as usize >= self.len() {
            return Tile::Wrap;
        }
        let (x, y) = (x as usize, y as usize);
        if x < self[y].len() {
            return self[y][x];
        }

        Tile::Wrap
    }
}

impl State {
    fn apply(mut self, mov: Move, map: &[Vec<Tile>]) -> State {
        match mov {
            Move::Forward(amount) => {
                for _ in 0..amount {
                    let (x, y) = match self.facing {
                        Facing::Up => {
                            let mut y = self.y as i32 - 1;

                            while matches!(map.at(self.x as i32, y), Tile::Wrap) {
                                if y < 0 {
                                    y = map.len() as i32
                                }
                                y -= 1;
                            }
                            (self.x as i32, y)
                        }
                        Facing::Down => {
                            let mut y = self.y as i32 + 1;

                            while matches!(map.at(self.x as i32, y), Tile::Wrap) {
                                if y >= map.len() as i32 {
                                    y = -1
                                }
                                y += 1;
                            }
                            (self.x as i32, y)
                        }
                        Facing::Left => {
                            let mut x = self.x as i32 - 1;

                            while matches!(map.at(x, self.y as i32), Tile::Wrap) {
                                if x < 0 {
                                    x = map[self.y].len() as i32
                                }
                                x -= 1;
                            }
                            (x, self.y as i32)
                        }
                        Facing::Right => {
                            let mut x = self.x as i32 + 1;

                            while matches!(map.at(x, self.y as i32), Tile::Wrap) {
                                if x >= map[self.y].len() as i32 {
                                    x = -1
                                }
                                x += 1;
                            }
                            (x, self.y as i32)
                        }
                    };
                    if matches!(map.at(x, y), Tile::Wall) {
                        break;
                    }
                    self.x = x as usize;
                    self.y = y as usize;
                }
            }
            Move::Turn(turn) => self.facing = self.facing.apply(turn),
        };
        self
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let (map, path) = {
        let lines = stdin().lock().lines();
        let mut lines = lines.flatten();
        let map = match lines.try_fold(Vec::new(), |mut map, line| {
            if line.is_empty() {
                return ControlFlow::Break(Ok(map));
            }

            let line = line
                .into_bytes()
                .into_iter()
                .try_fold(Vec::new(), |mut line, c| {
                    let tile = match c {
                        b' ' => Tile::Wrap,
                        b'.' => Tile::Open,
                        b'#' => Tile::Wall,
                        _ => return Err("unexpected tile"),
                    };
                    line.push(tile);
                    Ok(line)
                });

            let line = match line {
                Ok(line) => line,
                Err(err) => return ControlFlow::Break(Err(err)),
            };

            map.push(line);

            ControlFlow::Continue(map)
        }) {
            ControlFlow::Break(Ok(map)) => map,
            ControlFlow::Break(Err(err)) => return Err(err.into()),
            ControlFlow::Continue(_) => unreachable!(),
        };

        let path = lines
            .next()
            .ok_or("expected to find instructions")?
            .bytes()
            .try_fold(Vec::new(), |mut line, c| {
                match c {
                    b'0'..=b'9' => {
                        let n = (c - b'0') as usize;
                        if let Some(Move::Forward(p)) = &mut line.last_mut() {
                            *p = *p * 10 + n
                        } else {
                            line.push(Move::Forward((c - b'0') as usize))
                        }
                    }
                    b'L' => line.push(Move::Turn(Turn::Left)),
                    b'R' => line.push(Move::Turn(Turn::Right)),
                    _ => return Err::<_, Box<dyn Error>>(format!("invalid move: {}", c).into()),
                };

                Ok(line)
            })?;

        (map, path)
    };

    let state = path.into_iter().fold(
        State {
            x: 0,
            y: 0,
            facing: Facing::Right,
        },
        |state, mov| state.apply(mov, &map),
    );

    let score = (state.y + 1) * 1000 + (state.x + 1) * 4 + state.facing as usize;
    println!("{:?}", score);
    Ok(())
}
