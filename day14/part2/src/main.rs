use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::io::{stdin, BufRead};
use std::ops::Bound::{Excluded, Unbounded};

#[derive(Debug)]
enum Tile {
    Rock,
    Sand,
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let mut lines = lines.flatten();
    let mut max = None;

    let mut map = lines.try_fold(
        HashMap::<i32, BTreeMap<i32, Tile>>::new(),
        |mut map, line| {
            let mut positions = line.split(" -> ").map(|xy| {
                match xy.split_once(',').map(|parts| {
                    Ok::<(i32, i32), Box<dyn Error>>((parts.0.parse()?, parts.1.parse()?))
                }) {
                    Some(Ok((x, y))) => Ok((x, y)),
                    Some(Err(e)) => Err(e),
                    None => Err("invalid coordinates".into()),
                }
            });

            let mut source: (i32, i32) = positions
                .next()
                .ok_or("expected at least two positions")??;

            for destination in positions {
                let destination: (i32, i32) = destination?;
                let dx = (destination.0 - source.0).signum();
                let dy = (destination.1 - source.1).signum();
                if dx != 0 && dy != 0 {
                    return Err("diagonal moves not allowed".into());
                }
                loop {
                    let col = map.entry(source.0).or_default();
                    col.insert(source.1, Tile::Rock);

                    max = if let Some(max) = max {
                        Some(i32::max(max, source.1))
                    } else {
                        Some(source.1)
                    };

                    if source == destination {
                        break;
                    }
                    source = (source.0 + dx, source.1 + dy);
                }
            }

            Ok::<_, Box<dyn Error>>(map)
        },
    )?;

    let floor = max.ok_or("no lowest point found")? + 2;

    for col in map.values_mut() {
        col.insert(floor, Tile::Rock);
    }

    let result = (0..)
        .take_while(|_| {
            let mut sand = (500, 0);

            'main: loop {
                let y = match map.entry(sand.0) {
                    Entry::Occupied(col) => {
                        let next = col.get().range((Excluded(sand.1), Unbounded)).next();
                        match next {
                            None => unreachable!("sand should fall from above the floor"),
                            Some((y, _)) => *y,
                        }
                    }
                    Entry::Vacant(col) => {
                        col.insert(BTreeMap::from([(floor, Tile::Rock)]));
                        sand.1 = floor - 1;
                        break ;
                    },
                };
                for dx in [-1, 1] {
                    if map
                        .get(&(sand.0 + dx))
                        .and_then(|col| col.get(&y))
                        .is_none()
                    {
                        sand.0 += dx;
                        sand.1 = y;
                        continue 'main;
                    };
                }
                sand.1 = y - 1;
                break;
            }

            if sand == (500, 0) {
                return false
            }

            map.get_mut(&sand.0)
                .expect("column is supposed to exist")
                .insert(sand.1, Tile::Sand);
            true
        })
        .count() + 1;

    println!("{}", result);
    Ok(())
}
