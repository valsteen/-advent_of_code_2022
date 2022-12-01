use std::cmp::Reverse;
use std::error::Error;
use std::io::{stdin, BufRead};

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let mut calories = lines.flatten().try_fold(vec![0], |mut acc, line| {
        if line.is_empty() {
            acc.push(0)
        } else {
            *acc.last_mut().ok_or("expected integer")? += line.parse::<usize>()?
        }
        Ok::<_, Box<dyn Error>>(acc)
    })?;
    calories.sort_by_key(|c| Reverse(*c));
    let sum_top_three: usize = calories
        .chunks_exact(3)
        .next()
        .ok_or("not enough elements")?
        .iter()
        .sum();
    println!("{}", sum_top_three);
    Ok(())
}
