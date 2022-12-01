use std::error::Error;
use std::io::{stdin, BufRead};
use std::iter::once;

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let (max, _) = lines
        .flatten()
        .chain(once(String::new()))
        .try_fold((0, 0), |(max, current), line| {
            if line.is_empty() {
                if current > max {
                    Ok((current, 0))
                } else {
                    Ok((max, 0))
                }
            } else {
                Ok::<_, Box<dyn Error>>((max, current + line.parse::<usize>()?))
            }
        })?;
    println!("{}", max);
    Ok(())
}
