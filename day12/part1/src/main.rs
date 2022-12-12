use std::cmp::Ordering;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::ops::ControlFlow;

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();

    let (start, end, map) = lines.flatten().enumerate().try_fold(
        (None, None, Vec::new()),
        |(mut start, mut end, mut map), (y, line)| {
            map.push(
                line.bytes()
                    .enumerate()
                    .map(|(x, c)| match c {
                        b'S' => {
                            if start.is_none() {
                                start = Some((x, y))
                            } else {
                                Err("start is already set")?
                            }
                            Ok(0)
                        }
                        b'E' => {
                            if end.is_none() {
                                end = Some((x, y))
                            } else {
                                Err("end is already set")?
                            }
                            Ok(25)
                        }
                        b'a'..=b'z' => Ok(c - b'a'),
                        _ => Err("invalid height")?,
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            );
            Ok::<_, &str>((start, end, map))
        },
    )?;

    let start = start.ok_or("start was not set")?;
    let end = end.ok_or("end was not set")?;
    let distances = (0..map.len())
        .map(|y| {
            (0..map[y].len())
                .map(|x| if (x, y) == start { Some(0) } else { None })
                .collect()
        })
        .collect::<Vec<Vec<Option<usize>>>>();

    let result = (0..).try_fold(
        (vec![start], distances),
        |(mut to_visit, mut distances), _| {
            let (x, y) =
                if let Some((i, (x, y))) = to_visit.iter().copied().enumerate().min_by(
                    |(_, (x, y)), (_, (x1, y1))| match (distances[*y][*x], distances[*y1][*x1]) {
                        (Some(_), None) => Ordering::Less,
                        (None, Some(_)) => Ordering::Greater,
                        (d, d1) => d.cmp(&d1),
                    },
                ) {
                    to_visit.remove(i);
                    (x, y)
                } else {
                    return ControlFlow::Break(
                        distances[end.1][end.0].ok_or("end was not reached"),
                    );
                };

            let distance = if let Some(distance) = distances[y][x] {
                distance
            } else {
                return ControlFlow::Break(Err("current distance is unknown"));
            };

            for (dx, dy) in [(-1, 0), (1, 0), (0, 1), (0, -1)] {
                if let (Some(x1), Some(y1)) = (
                    usize::try_from(x as i32 + dx)
                        .ok()
                        .filter(|x| *x < map[0].len()),
                    usize::try_from(y as i32 + dy)
                        .ok()
                        .filter(|y| *y < map.len()),
                ) {
                    if map[y1][x1] <= map[y][x] + 1
                        && distances[y1][x1].filter(|d1| *d1 < distance + 1).is_none()
                    {
                        distances[y1][x1] = Some(distance + 1);
                        if !to_visit.contains(&(x1, y1)) {
                            to_visit.push((x1, y1))
                        }
                    }
                }
            }

            ControlFlow::Continue((to_visit, distances))
        },
    );

    match result {
        ControlFlow::Continue(_) => unreachable!(),
        ControlFlow::Break(Ok(distance)) => println!("distance found: {}", distance),
        ControlFlow::Break(Err(e)) => Err(e)?,
    }
    Ok(())
}
