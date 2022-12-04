use std::error::Error;
use std::io::{stdin, BufRead};

fn main() -> Result<(), Box<dyn Error>> {
    let re = regex::Regex::new(r"^(\d+)-(\d+),(\d+)-(\d+)$")?;
    let lines = stdin().lock().lines();
    let result = lines.flatten().try_fold(0, |acc, line| {
        let captures = re.captures(&line).ok_or("invalid input")?;
        let mut line = captures
            .iter()
            .skip(1)
            .flatten()
            .map(|m| str::parse::<usize>(m.as_str()));
        let mut next = move || line.next().ok_or("expected more elements");
        let (left, right) = ((next()??, next()??), (next()??, next()??));
        if (left.0 >= right.0 && left.1 <= right.1) || (right.0 >= left.0 && right.1 <= left.1) {
            Ok::<_, Box<dyn Error>>(acc + 1)
        } else {
            Ok(acc)
        }
    })?;
    println!("{}", result);
    Ok(())
}
