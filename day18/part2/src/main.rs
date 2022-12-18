use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::iter::once_with;

/*
two approaches:
- scan line by line. for all cubes, all non-cube neighbours are marked as air
- starting from next line, every empty space touching air becomes air .. but then we need several
  passes until the air are all marked

- make a visitor. start from a free location next to a cube
- iterate the 8 neighbours
  - if at least one side is touching :
    - mark all neighbours : if cube, add to surface counter, if air, ignore, if unmarked, mark as air and put in a queue
 */

// it's not 2028 ... ?
// not 2030 --
/*
That's not the right answer; your answer is too high.
If you're stuck, make sure you're using the full input data; there are also some general tips on the about page,
or you can ask for hints on the subreddit. Please wait one minute before trying again. (You guessed 2030.) [Return to Day 18]
 */

fn edge_neighbours(cube: [i8; 3]) -> [[i8; 3]; 14] {
    [
        [cube[0] - 1, cube[1], cube[2]],
        [cube[0] - 1, cube[1] - 1, cube[2]],
        [cube[0] - 1, cube[1] + 1, cube[2]],
        [cube[0] - 1, cube[1], cube[2] - 1],
        [cube[0] - 1, cube[1], cube[2] + 1],
        [cube[0] + 1, cube[1], cube[2]],
        [cube[0] + 1, cube[1] - 1, cube[2]],
        [cube[0] + 1, cube[1] + 1, cube[2]],
        [cube[0] + 1, cube[1], cube[2] - 1],
        [cube[0] + 1, cube[1], cube[2] + 1],
        [cube[0], cube[1] - 1, cube[2]],
        [cube[0], cube[1] + 1, cube[2]],
        [cube[0], cube[1], cube[2] - 1],
        [cube[0], cube[1], cube[2] + 1],
    ]
}

fn neighbours(cube: [i8; 3]) -> [[i8; 3]; 6] {
    [
        [cube[0] - 1, cube[1], cube[2]],
        [cube[0] + 1, cube[1], cube[2]],
        [cube[0], cube[1] - 1, cube[2]],
        [cube[0], cube[1] + 1, cube[2]],
        [cube[0], cube[1], cube[2] - 1],
        [cube[0], cube[1], cube[2] + 1],
    ]
}

fn exposed(cubes: Vec<[i8; 3]>) -> usize {
    let Some(mut first) = cubes.iter().max().copied() else {
        return 0
    };
    first[0] += 1;
    println!("start : {:?}", first);
    let cubes = HashSet::<_, RandomState>::from_iter(cubes);
    let mut air = HashSet::new();

    let mut queue = vec![(first, true)];
    let mut exposed = 0;
    while let Some((location, should_dive)) = queue.pop() {
        if air.contains(&location) {
            continue;
        }

        if cubes.contains(&location) {
            exposed += 1;
            continue;
        }
        air.insert(location);

        let neighbours = neighbours(location);
        let touches = edge_neighbours(location).iter().any(|cube|  cubes.contains(cube));
        if !touches && !should_dive {
            continue;
        }
        println!("{:?} {:?}", location, neighbours);
        //assert_ne!([2,2,5], location);
        queue.extend(neighbours.into_iter().map(|cube| (cube, touches)));
    }
    exposed
}
//
// fn rotations<T>(cubes: &'_ T) -> impl Iterator<Item = T> + '_
// where
//     T: 'static + AsMut<[[i8; 3]]> + Clone,
// {
//     once_with(|| cubes.clone())
//         .chain(once_with(|| {
//             let mut cubes = cubes.clone();
//             cubes
//                 .as_mut()
//                 .iter_mut()
//                 .for_each(|c| *c = [c[2], c[0], c[1]]);
//             cubes
//         }))
//         .chain(once_with(|| {
//             let mut cubes = cubes.clone();
//             cubes
//                 .as_mut()
//                 .iter_mut()
//                 .for_each(|c| *c = [c[1], c[2], c[0]]);
//             cubes
//         }))
// }
//
// fn touches(left: [i8; 3], right: [i8; 3]) -> bool {
//     rotations(&[left, right]).any(|cubes| {
//         cubes[0][0] == cubes[1][0]
//             && cubes[0][1] == cubes[1][1]
//             && cubes[0][2].abs_diff(cubes[1][2]) == 1
//     })
// }

fn main() -> Result<(), Box<dyn Error>> {
    let cubes: Vec<[i8; 3]> = {
        let lines = stdin().lock().lines();
        lines
            .flatten()
            .map(|line| {
                line.split(',')
                    .map(str::parse)
                    .enumerate()
                    .try_fold([0; 3], |mut acc, (i, v)| {
                        if i > 2 {
                            return Err("trailing content".into());
                        }
                        acc[i] = v?;
                        Ok(acc)
                    })
            })
            .collect::<Result<_, Box<dyn Error>>>()?
    };

    let groups = cubes
        .into_iter()
        .fold(vec![], |mut acc: Vec<Vec<[i8; 3]>>, cube| {
            let edge_neighbours = edge_neighbours(cube);

            let mut merge_with = acc.iter_mut().filter_map(|group| {
                group
                    .iter()
                    .any(|group_cube| edge_neighbours.contains(group_cube))
                    .then_some(group)
            });

            if let Some(first) = merge_with.next() {
                first.push(cube);

                for merger in merge_with {
                    first.append(merger);
                }

                acc.retain(|cubes| !cubes.is_empty());
            } else {
                acc.push(vec![cube]);
            }
            acc
        });

    for group in &groups {
        println!("{:?}", group);
    }

    let result = groups.into_iter().map(exposed).sum::<usize>();
    println!("{}", result);

    Ok(())
}
