use std::convert::identity;
use std::error::Error;
use std::io::{stdin, BufRead};

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let trees = lines
        .map(|line| {
            line?
                .chars()
                .map(|c| Ok(c.to_digit(10).ok_or("invalid character")? as i8))
                .collect()
        })
        .collect::<Result<Vec<Vec<i8>>, Box<dyn Error>>>()?;

    if trees.is_empty() || trees[0].is_empty() {
        Err("empty list")?
    }

    let mut visibility = trees
        .iter()
        .map(|row| {
            row.iter()
                .fold((-1, Vec::new()), |(mut max, mut row), height| {
                    if *height > max {
                        row.push(true);
                        max = *height;
                    } else {
                        row.push(false)
                    }
                    (max, row)
                })
                .1
        })
        .collect::<Vec<Vec<bool>>>();

    let mut get_max_adjust = |max, x: usize, y: usize| {
        if trees[y][x] > max {
            visibility[y][x] = true;
            trees[y][x]
        } else {
            max
        }
    };

    for y in 0..trees.len() {
        (0..trees[0].len())
            .rev()
            .fold(-1, |max, x| get_max_adjust(max, x, y));
    }

    for x in 0..trees.len() {
        (0..trees[0].len()).fold(-1, |max, y| get_max_adjust(max, x, y));
        (0..trees[0].len())
            .rev()
            .fold(-1, |max, y| get_max_adjust(max, x, y));
    }

    let result = visibility.iter().map(|v| v.iter().filter(|v| **v).count()).sum::<usize>();
    println!("{}", result);

    Ok(())
}
