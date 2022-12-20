use std::{
    borrow::BorrowMut,
    cell::RefCell,
    error::Error,
    io::{stdin, BufRead},
    ops::Deref,
    rc::Rc,
};

fn main() -> Result<(), Box<dyn Error>> {
    let input = {
        let lines = stdin().lock().lines();
        lines
            .enumerate()
            .map(|(i, s)| Ok(Rc::new(RefCell::new((s?.parse()?, i)))))
            .collect::<Result<Vec<Rc<RefCell<(i32, usize)>>>, Box<dyn Error>>>()?
    };

    let mut destinations = input.clone();

    let zero = input
        .iter()
        .find(|n| RefCell::borrow(n).deref().0 == 0)
        .unwrap()
        .clone();

    for n in 0..input.len() {
        let current = input[n].clone();
        let (delta, old_pos) = *current.as_ref().borrow();

        let new_pos = if delta < 0 {
            let mut delta = delta;
            while delta < 0 {
                delta += input.len() as i32 - 1
            }

            let new_pos = (old_pos + delta as usize) % input.len();

            new_pos as usize
        } else {
            let delta = delta % (input.len() as i32 - 1);
            (old_pos + delta as usize) % input.len()
        };

        if old_pos == new_pos {
            continue;
        }

        let mut pull_index = old_pos;
        loop {
            let next_index = (pull_index + 1) % input.len();
            let item_to_move = destinations[next_index].borrow_mut();

            item_to_move.as_ref().borrow_mut().1 = pull_index;
            destinations[pull_index] = item_to_move.clone();

            if next_index == new_pos {
                current.as_ref().borrow_mut().1 = new_pos;
                destinations[next_index] = current.clone();
                break;
            }

            pull_index = (pull_index + 1) % input.len()
        }
    }

    let start = zero.as_ref().borrow().1;

    let result = [1000usize, 2000, 3000]
        .into_iter()
        .map(|delta| {
            let pos = (delta + start) % input.len();
            let current = destinations[pos].as_ref().borrow().0;
            current
        })
        .sum::<i32>();

    println!("{}", result);
    Ok(())
}
