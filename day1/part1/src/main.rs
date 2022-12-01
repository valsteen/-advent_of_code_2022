use std::error::Error;
use std::io::{BufRead, stdin};

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let calories = lines.flatten().try_fold(vec![0], |mut acc, line| {
        if line.is_empty() {
            acc.push(0)
        } else {
            *acc.last_mut().ok_or("expected integer")? += line.parse::<usize>()?
        }
        Ok::<_, Box<dyn Error>>(acc)
    })?;
    println!("{}", calories.iter().max().ok_or("cannot find maximum")?);
    Ok(())
}
