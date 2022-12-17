use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::{BufRead, stdin};

#[derive(Copy, Clone)]
enum Jet {
    Left,
    Right,
}

impl From<Jet> for char {
    fn from(jet: Jet) -> Self {
        match jet {
            Jet::Left => '<',
            Jet::Right => '>',
        }
    }
}

impl Display for Jet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&char::from(*self), f)
    }
}

impl Debug for Jet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&char::from(*self), f)
    }
}

impl TryFrom<u8> for Jet {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'<' => Ok(Self::Left),
            b'>' => Ok(Self::Right),
            _ => Err("invalid character"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Shape {
    Horizontal,
    Plus,
    L,
    Vertical,
    Square,
}

impl From<Shape> for Vec<(usize, usize)> {
    fn from(shape: Shape) -> Self {
        match shape {
            Shape::Horizontal => vec![(0, 0), (1, 0), (2, 0), (3, 0)],
            Shape::Plus => vec![(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)],
            Shape::L => vec![(0, 0), (1, 0), (2, 0), (2, 1), (2, 2)],
            Shape::Vertical => vec![(0, 0), (0, 1), (0, 2), (0, 3)],
            Shape::Square => vec![(0, 0), (1, 0), (0, 1), (1, 1)],
        }
    }
}

struct Repeater<'a, I>(&'a [I], usize);

impl<'a, I> Iterator for Repeater<'a, I> {
    type Item = &'a I;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.1;
        self.1 = (self.1 + 1) % self.0.len();
        Some(&self.0[current])
    }
}

impl<'a, I> From<&'a [I]> for Repeater<'a, I> {
    fn from(s: &'a [I]) -> Self {
        Self(s, 0)
    }
}

const WIDTH: usize = 7;
const ITERATIONS: usize = 2022;
const START_HEIGHT: usize = 3;

trait Blocks<I>
where
    Self: Sized,
{
    fn apply(self, j: Jet, bottom: &Self) -> Result<Self, Self>;
    fn down(self, bottom: &Self) -> Result<Self, Self>;
    fn merge_into(self, blocks: Self) -> Self;
    fn max_height(&self) -> I;
}

trait AnyResult<I> {
    fn any(self) -> I;
}

impl<I> AnyResult<I> for Result<I, I> {
    fn any(self) -> I {
        match self {
            Ok(res) => res,
            Err(res) => res,
        }
    }
}

impl Blocks<usize> for Vec<(usize, usize)> {
    fn apply(mut self, d: Jet, bottom: &Self) -> Result<Self, Self> {
        match d {
            Jet::Left => {
                if self
                    .iter()
                    .all(|(x, y)| *x > 0 && !bottom.contains(&(*x - 1, *y)))
                {
                    self.iter_mut().for_each(|(x, _)| {
                        *x -= 1;
                    });
                    Ok(self)
                } else {
                    Err(self)
                }
            }
            Jet::Right => {
                if self
                    .iter()
                    .all(|(x, y)| *x + 1 < WIDTH && !bottom.contains(&(*x + 1, *y)))
                {
                    self.iter_mut().for_each(|(x, _)| {
                        *x += 1;
                    });
                    Ok(self)
                } else {
                    Err(self)
                }
            }
        }
    }

    fn down(mut self, other: &Self) -> Result<Self, Self> {
        if self
            .iter()
            .all(|(x, y)| *y > 0 && !other.contains(&(*x, *y - 1)))
        {
            self.iter_mut().for_each(|(_, y)| {
                *y -= 1;
            });
            Ok(self)
        } else {
            Err(self)
        }
    }

    fn merge_into(mut self, blocks: Vec<(usize, usize)>) -> Self {
        for (x, y) in blocks {
            self.push((x, y));
        }
        self
    }

    fn max_height(&self) -> usize {
        self.iter().map(|(_, y)| *y).max().unwrap_or_default()
    }
}

#[allow(dead_code)]
fn display(me: &Vec<(usize, usize)>, block: &Vec<(usize, usize)>, jet: Jet) {
    print!("\x1B[2J\x1B[1;1H");
    let mut displayed = 0;
    for y in (0..=block.max_height().max(me.max_height())).rev().take(50) {
        let output = (0..WIDTH)
            .map(|x| {
                if me.contains(&(x, y)) {
                    displayed += 1;
                    '#'
                } else if block.contains(&(x, y)) {
                    displayed += 1;
                    '@'
                } else {
                    '.'
                }
            })
            .collect::<String>();
        println!("{}", output);
        if displayed == me.len() + block.len() {
            break;
        }
    }
    println!("{:?} {}", block, jet);
}

fn main() -> Result<(), Box<dyn Error>> {
    let patterns: Vec<Jet> = {
        let lines = stdin().lock().lines();
        let mut lines = lines.flatten();
        let patterns = lines
            .next()
            .ok_or("expected input")?
            .bytes()
            .map(Jet::try_from)
            .collect::<Result<_, _>>()?;
        if lines.next().is_some() {
            return Err("trailing content".into());
        }
        patterns
    };

    let shapes = Repeater::from(
        [
            Shape::Horizontal,
            Shape::Plus,
            Shape::L,
            Shape::Vertical,
            Shape::Square,
        ]
        .as_slice(),
    );

    let mut jets = Repeater::from(patterns.as_slice());

    let mut bottom = Vec::from_iter((0..WIDTH).map(|x| (x, 0usize)));

    for &shape in shapes.take(ITERATIONS) {
        let mut block = Vec::from(shape);
        let min_y = bottom.max_height() + START_HEIGHT + 1;
        block.iter_mut().for_each(|(x, y)| {
            *x += 2;
            *y += min_y;
        });

        for &jet in &mut jets {
            block = block.apply(jet, &bottom).any();
            block = match block.down(&bottom) {
                Ok(block) => block,
                Err(block) => {
                    bottom = bottom.merge_into(block);
                    break;
                }
            }
        }
    }

    println!("{}", bottom.max_height());

    Ok(())
}
