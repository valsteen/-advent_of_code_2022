use std::error::Error;
use std::io::{stdin, BufRead};
use std::ops::ControlFlow;

trait ControlFlowValue<T> {
    fn value(self) -> Option<T>;
}

impl ControlFlowValue<usize> for ControlFlow<usize,usize> {
    fn value(self) -> Option<usize> {
        let result = match self {
            ControlFlow::Break(v) | ControlFlow::Continue(v) => v
        };
        (result != 0).then_some(result)
    }
}

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

    let result = trees.iter().enumerate().flat_map(|(y, row)| {
        let trees = &trees;
        row.iter().enumerate().map(move | (x, height)| {
            let score_col = || (0..y).rev().try_fold(0, |count, y1| {
                if trees[y1][x] < *height {
                    ControlFlow::Continue(count+1)
                } else {
                    ControlFlow::Break(count+1)
                }
            }).value();
            let score_col2 = || (y+1..trees.len()).try_fold(0, |count, y1| {
                if trees[y1][x] < *height {
                    ControlFlow::Continue(count+1)
                } else {
                    ControlFlow::Break(count+1)
                }
            }).value();
            let score_row = || (0..x).rev().try_fold(0, |count, x1| {
                if trees[y][x1] < *height {
                    ControlFlow::Continue(count+1)
                } else {
                    ControlFlow::Break(count+1)
                }
            }).value();
            let score_row2 = || (x+1..trees[0].len()).try_fold(0, |count, x1| {
                if trees[y][x1] < *height {
                    ControlFlow::Continue(count+1)
                } else {
                    ControlFlow::Break(count+1)
                }
            }).value();

            score_col().
                and_then(|res| Some(res * score_col2()?)).
                and_then(|res| Some(res * score_row()?)).
                and_then(|res| Some(res * score_row2()?)).unwrap_or_default()
        })
    }).max().ok_or("cannot find max")? ;

    println!("{:?}", result);

    Ok(())
}
