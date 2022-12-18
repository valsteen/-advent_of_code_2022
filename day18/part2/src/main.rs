use std::cmp::Reverse;
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::error::Error;
use std::io::{stdin, BufRead};

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

fn exposed(cubes: Vec<[i8; 3]>, trapped: &mut HashSet<[i8; 3]>) -> usize {
    let Some(mut first) = cubes.iter().max().copied() else {
        return 0
    };
    first[0] += 1;
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
        let touches = edge_neighbours(location)
            .iter()
            .any(|cube| cubes.contains(cube));
        if !touches && !should_dive {
            continue;
        }
        queue.extend(neighbours.into_iter().map(|cube| (cube, touches)));
    }

    let mut trapped_queue = cubes
        .iter()
        .flat_map(|cube| neighbours(*cube))
        .collect::<Vec<_>>();

    while let Some(location) = trapped_queue.pop() {
        if cubes.contains(&location) || trapped.contains(&location) || air.contains(&location) {
            continue;
        }
        trapped.insert(location);
        trapped_queue.extend(neighbours(location))
    }

    exposed
}

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

    let mut groups = cubes
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

    groups.sort_by_key(|g| Reverse(g.len()));
    let mut trapped = HashSet::new();

    let result = groups
        .into_iter()
        .map(|cubes| {
            if cubes.iter().any(|cube| trapped.contains(cube)) {
                0
            } else {
                exposed(cubes, &mut trapped)
            }
        })
        .sum::<usize>();
    println!("{}", result);

    Ok(())
}
