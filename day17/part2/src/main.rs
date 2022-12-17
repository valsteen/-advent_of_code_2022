use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::{stdin, BufRead};
use std::ops::{Deref, DerefMut};
use std::vec::IntoIter;

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

impl From<Shape> for Blocks {
    fn from(shape: Shape) -> Self {
        Self::from(match shape {
            Shape::Horizontal => vec![(0, 0), (1, 0), (2, 0), (3, 0)],
            Shape::Plus => vec![(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)],
            Shape::L => vec![(0, 0), (1, 0), (2, 0), (2, 1), (2, 2)],
            Shape::Vertical => vec![(0, 0), (0, 1), (0, 2), (0, 3)],
            Shape::Square => vec![(0, 0), (1, 0), (0, 1), (1, 1)],
        })
    }
}

impl From<Vec<(usize, usize)>> for Blocks {
    fn from(vec: Vec<(usize, usize)>) -> Self {
        Self {
            blocks: vec,
            base: 0,
        }
    }
}

impl IntoIterator for Blocks {
    type Item = (usize, usize);
    type IntoIter = IntoIter<(usize, usize)>;

    fn into_iter(self) -> Self::IntoIter {
        self.blocks.into_iter()
    }
}

struct Repeater<'a, I>(&'a [I], usize);

impl<'a, I> Iterator for Repeater<'a, I> {
    type Item = (usize, &'a I);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.1;
        self.1 = (self.1 + 1) % self.0.len();
        Some((current, &self.0[current]))
    }
}

impl<'a, I> From<&'a [I]> for Repeater<'a, I> {
    fn from(s: &'a [I]) -> Self {
        Self(s, 0)
    }
}

const WIDTH: usize = 7;
const ITERATIONS: usize = 1000000000000;
const START_HEIGHT: usize = 3;

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

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct Blocks {
    blocks: Vec<(usize, usize)>,
    base: usize,
}

impl Deref for Blocks {
    type Target = Vec<(usize, usize)>;

    fn deref(&self) -> &Self::Target {
        &self.blocks
    }
}

impl Debug for Blocks {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.blocks, f)
    }
}

impl DerefMut for Blocks {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.blocks
    }
}

impl Blocks {
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

    fn merge_into(mut self, blocks: Blocks) -> (Self, Option<Self>) {
        let mut remove_at = None;

        for (x, y) in blocks {
            self.push((x, y));

            if y > 1 && remove_at.is_none() {
                let mut found = [false; WIDTH];
                self.iter()
                    .filter(|(_, sy)| *sy == y || *sy + 1 == y)
                    .for_each(|(x, _)| {
                        found[*x] = true;
                    });

                if found.into_iter().all(|b| b) {
                    remove_at = Some(y);
                }
            }
        }

        if let Some(remove_at) = remove_at {
            let mut clone = self.clone();
            clone.base = 0;

            self.base = self.base + remove_at - 1;
            self.retain_mut(|(_, sy)| {
                if *sy >= remove_at - 1 {
                    *sy -= remove_at - 1;
                    true
                } else {
                    false
                }
            });
            return (self, Some(clone));
        }
        (self, None)
    }

    fn max_height(&self) -> usize {
        self.iter().map(|(_, y)| *y).max().unwrap_or_default() + self.base
    }

    #[allow(dead_code)]
    fn min_height(&self) -> usize {
        self.iter().map(|(_, y)| *y).min().unwrap_or_default() + self.base
    }
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

    let mut bottom: Blocks = Vec::from_iter((0..WIDTH).map(|x| (x, 0usize))).into();
    let mut clones = HashMap::new();
    let mut iteration = 0;

    for (shape_index, &shape) in shapes.take(ITERATIONS) {
        iteration += 1;
        if iteration > ITERATIONS {
            break;
        }
        let mut block = Blocks::from(shape);

        let min_y = bottom.max_height() + START_HEIGHT + 1 - bottom.base;
        block.iter_mut().for_each(|(x, y)| {
            *x += 2;
            *y += min_y;
        });

        for (jet_index, &jet) in &mut jets {
            block = block.apply(jet, &bottom).any();
            block = match block.down(&bottom) {
                Ok(block) => block,
                Err(block) => {
                    let (mut new_bottom, clone) = bottom.merge_into(block);
                    if let Some(clone) = clone {
                        if let Some((previous_iteration, previous_height)) = clones.insert(
                            (clone, jet_index, shape_index),
                            (iteration, new_bottom.max_height()),
                        ) {
                            let loop_length = iteration - previous_iteration;
                            let loop_height = new_bottom.max_height() - previous_height;
                            let repeat = (ITERATIONS - iteration) / loop_length;
                            new_bottom.base += loop_height * repeat;
                            iteration += repeat * loop_length;
                        };
                    }

                    bottom = new_bottom;
                    break;
                }
            }
        }
    }

    println!("{}", bottom.max_height());

    Ok(())
}
