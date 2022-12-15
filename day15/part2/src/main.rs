use std::collections::{BTreeSet, HashSet};
use std::io::{stdin, BufRead};
use std::ops::Neg;
use std::str::FromStr;

use nom::error::VerboseErrorKind;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{all_consuming, map_res},
    error::{ContextError, FromExternalError, ParseError, VerboseError},
    sequence::tuple,
    Finish, IResult,
};

trait Number: FromStr + Neg<Output = Self> {}

impl<T> Number for T where T: FromStr + Neg<Output = Self> {}

trait ExpressionParseError<'a, T: FromStr>:
    ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, <T as FromStr>::Err>
{
}
impl<'a, T: FromStr> ExpressionParseError<'a, T> for VerboseError<&'a str> {}

fn number<'a, E: ExpressionParseError<'a, T>, T: Number>(i: &'a str) -> IResult<&'a str, T, E> {
    alt((
        map_res(digit1, |s: &str| s.parse::<T>()),
        map_res(tuple((tag("-"), digit1::<&str, _>)), |(_, s)| {
            s.parse::<T>().map(|v| -v)
        }),
    ))(i)
}

trait Tiles {
    fn exclude(&mut self, tile: Tile);
    fn add(&mut self, tile: Tile) -> bool;
}

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
struct Tile {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Tile {
    fn valid(&self) -> bool {
        self.x1 <= self.x2 && self.y1 <= self.y2
    }
}

impl Tiles for HashSet<Tile> {
    fn exclude(&mut self, tile: Tile) {
        let mut backing = HashSet::new();

        self.retain(|this| {
            let intersects = backing.add(Tile {
                x1: this.x1,
                y1: this.y1,
                x2: tile.x1.min(this.x2),
                y2: tile.y1.min(this.y2),
            }) || backing.add(Tile {
                x1: this.x1,
                y1: tile.y1.max(this.y1),
                x2: tile.x1.min(this.x2),
                y2: tile.y2.min(this.y2),
            }) || backing.add(Tile {
                x1: this.x1,
                y1: tile.y2.max(this.y1),
                x2: tile.x1.min(this.x2),
                y2: this.y2,
            }) || backing.add(Tile {
                x1: tile.x1.max(this.x1),
                y1: this.y1,
                x2: tile.x2.min(this.x2),
                y2: tile.y1.min(this.y2),
            }) || backing.add(Tile {
                x1: tile.x1.max(this.x1),
                y1: tile.y2.max(this.y1),
                x2: tile.x2.min(this.x2),
                y2: this.y2,
            }) || backing.add(Tile {
                x1: tile.x2.max(this.x1),
                y1: this.y1,
                x2: this.x2,
                y2: tile.y1.min(this.y2),
            }) || backing.add(Tile {
                x1: tile.x2.max(this.x1),
                y1: tile.y1.max(this.y1),
                x2: this.x2,
                y2: tile.y2.min(this.y2),
            }) || backing.add(Tile {
                x1: tile.x2.max(this.x1),
                y1: tile.y2.max(this.y1),
                x2: this.x2,
                y2: this.y2,
            });
            !intersects
        });

        self.extend(backing)
    }

    fn add(&mut self, tile: Tile) -> bool {
        if tile.valid() {
            self.insert(tile);
            true
        } else {
            false
        }
    }
}

fn line<'a, T: Number, E: ExpressionParseError<'a, T>>(
    i: &'a str,
) -> IResult<&'a str, (T, T, T, T), E> {
    let (_, (_, x1, _, y1, _, x2, _, y2)) = all_consuming(tuple((
        tag("Sensor at x="),
        number,
        tag(", y="),
        number,
        tag(": closest beacon is at x="),
        number,
        tag(", y="),
        number,
    )))(i)?;

    Ok((i, (x1, y1, x2, y2)))
}

fn main() {
    let lines = stdin().lock().lines();
    let lines = lines.flatten().enumerate().map(|(n, s)| {
        line::<i32, VerboseError<_>>(&s)
            .finish()
            .map(|(_, line)| line)
            .map_err(|err| {
                (
                    n,
                    err.errors
                        .into_iter()
                        .map(|(remain, e)| (remain.to_string(), e))
                        .collect::<Vec<(String, VerboseErrorKind)>>(),
                )
            })
    });

    let mut beacons = HashSet::new();

    let mut tiles = HashSet::from([Tile {
        x1: 0,
        y1: 0,
        x2: 20,
        y2: 20,
    }]);

    for line in lines {
        let (x1, y1, x2, y2) = match line {
            Ok(line) => line,
            Err((n, errs)) => {
                println!("Error at line {}:", n);
                for (input, r) in errs {
                    println!("\t'{}': {:?}", input, r)
                }
                return;
            }
        };

        beacons.insert((x1, y1));
        beacons.insert((x2, y2));

        tiles.exclude(Tile{
            x1,
            y1,
            x2,
            y2,
        });
    }

    println!("{}", intervals.count() - beacons.len());
}
