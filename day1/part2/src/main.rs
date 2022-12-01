use std::error::Error;
use std::io::{stdin, BufRead};
use std::iter::once;

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let (top_three, _) = lines.flatten().chain(once(String::new())).try_fold(
        (vec![], 0),
        |(mut result, current), line| {
            if line.is_empty() {
                let pos = result
                    .binary_search_by(|e| current.cmp(e))
                    .unwrap_or_else(|e| e);
                result.insert(pos, current);
                result.truncate(3);
                Ok((result, 0))
            } else {
                Ok::<_, Box<dyn Error>>((result, current + line.parse::<usize>()?))
            }
        },
    )?;
    println!("{}", top_three.iter().sum::<usize>());
    Ok(())
}
