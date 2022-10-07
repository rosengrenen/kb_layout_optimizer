use lazy_static::lazy_static;
use rand::{seq::SliceRandom, thread_rng, Rng};
use rayon::{
    prelude::{IntoParallelRefMutIterator, ParallelIterator},
    slice::ParallelSlice,
};

#[derive(Clone, Debug)]
struct Keyboard {
    keys: [[char; 3]; 10],
}

impl Default for Keyboard {
    fn default() -> Self {
        Self {
            keys: [['-'; 3]; 10],
        }
    }
}

impl Keyboard {
    fn print(&self) {
        for y in 0..3 {
            for x in 0..10 {
                print!("{} ", self.keys[x][y]);
            }

            println!();
        }
    }

    fn key_pos(&self, key: char) -> (usize, usize) {
        for x in 0..10 {
            for y in 0..3 {
                if self.keys[x][y] == key {
                    return (x, y);
                }
            }
        }

        unreachable!();
    }

    fn key_finger(&self, key: char) -> usize {
        let (x, _) = self.key_pos(key);
        match x {
            4 => 3,
            5 => 6,
            n => n,
        }
    }
}

const CHARS: [char; 30] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', ',', '.', '?', ';',
];
const MOST_COMMON: [char; 8] = ['e', 't', 'a', 'o', 'i', 'n', 's', 'r'];
lazy_static! {
    static ref BIGRAMS: Vec<(String, i32)> = include_str!("../data/english_bigrams_1.txt")
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (bigram, frequency) = line.split_once(' ').unwrap();
            (bigram.to_lowercase(), frequency.parse::<i32>().unwrap())
        })
        .collect();
}
const QWERTY: Keyboard = Keyboard {
    keys: [
        ['q', 'a', 'z'],
        ['w', 's', 'x'],
        ['e', 'd', 'c'],
        ['r', 'f', 'v'],
        ['t', 'g', 'b'],
        ['y', 'h', 'n'],
        ['u', 'j', 'm'],
        ['i', 'k', ','],
        ['o', 'l', '.'],
        ['p', ';', '?'],
    ],
};
const DVORAK: Keyboard = Keyboard {
    keys: [
        ['"', 'a', ';'],
        [',', 'o', 'q'],
        ['.', 'e', 'j'],
        ['p', 'u', 'k'],
        ['y', 'i', 'x'],
        ['f', 'd', 'b'],
        ['g', 'h', 'm'],
        ['c', 't', 'w'],
        ['r', 'n', 'v'],
        ['l', 's', 'z'],
    ],
};

// Settings
const POPULATION_SIZE: usize = 1000;
const GENERATIONS: usize = 10000;
const TOURNAMENT_PROBABILTY: f64 = 0.8;
const TOURNAMENT_SIZE: usize = 5;
const MUTATION_PROBABILITY: f64 = 0.1;

fn generate_individual() -> Keyboard {
    let mut chars = CHARS.to_vec();
    chars.shuffle(&mut thread_rng());
    let mut most_common = MOST_COMMON.to_vec();
    most_common.shuffle(&mut thread_rng());
    let mut keyboard = Keyboard::default();

    for i in 0..10 {
        if i == 4 || i == 5 {
            continue;
        }

        let a = most_common.pop().unwrap();
        keyboard.keys[i][1] = a;
        chars.retain(|&b| b != a);
    }

    for i in 0..10 {
        keyboard.keys[i][0] = chars.pop().unwrap();
        keyboard.keys[i][2] = chars.pop().unwrap();
    }

    keyboard.keys[4][1] = chars.pop().unwrap();
    keyboard.keys[5][1] = chars.pop().unwrap();

    keyboard
}

fn generate_population() -> Vec<Keyboard> {
    let mut keyboards = Vec::with_capacity(POPULATION_SIZE);
    for i in 0..POPULATION_SIZE {
        keyboards.push(generate_individual());
    }

    keyboards
}

fn evaluate_individual(individual: &Keyboard) -> f64 {
    let mut fitness = 0.0;
    for (bigram, freq) in BIGRAMS.iter() {
        let freq = *freq;
        let mut chars = bigram.chars();
        let first_c = chars.next().unwrap();
        let second_c = chars.next().unwrap();
        let (first_x, first_y) = individual.key_pos(first_c);
        let (second_x, second_y) = individual.key_pos(second_c);
        let first_finger = individual.key_finger(first_c);
        let second_finger = individual.key_finger(second_c);
        if first_finger == second_finger {
            let first_x = first_x as f64;
            let first_y = first_y as f64;
            let second_x = second_x as f64;
            let second_y = second_y as f64;
            fitness += ((first_x - second_x).powi(2) + (first_y - second_y).powi(2)) * freq as f64;
        } else {
            let mut factor = 0;
            factor += match first_y {
                0 => 1,
                2 => 5,
                _ => 0,
            };
            factor += match second_y {
                0 => 1,
                2 => 5,
                _ => 0,
            };
            fitness += factor as f64 * freq as f64;
        }
    }

    fitness
}

fn tournament_selection(fitnesses: &[f64]) -> usize {
    let mut rng = rand::thread_rng();
    let mut indices = Vec::with_capacity(TOURNAMENT_SIZE);
    for _ in 0..TOURNAMENT_SIZE {
        indices.push(rng.gen::<usize>() % fitnesses.len());
    }

    while indices.len() > 1 {
        let index = indices.pop().unwrap();
        if rng.gen::<f64>() < TOURNAMENT_PROBABILTY {
            // lower is better
            if fitnesses[index] < fitnesses[indices[indices.len() - 1]] {
                return index;
            }
        }
    }

    indices.pop().unwrap()
}

fn cross(first_individual: &Keyboard, second_individual: &Keyboard) -> (Keyboard, Keyboard) {
    let crossover_point = rand::random::<usize>() % 30;
    let mut first_new_individual = Keyboard::default();
    let mut second_new_individual = Keyboard::default();
    for i in 0..30 {
        if i < crossover_point {
            first_new_individual.keys[i / 3][i % 3] = first_individual.keys[i / 3][i % 3];
        } else {
            second_new_individual.keys[i / 3][i % 3] = second_individual.keys[i / 3][i % 3];
        }
    }

    // Fill in missing in first
    let mut first_missing_most_common = MOST_COMMON.to_vec();
    let mut first_missing_chars = CHARS.to_vec();
    first_missing_chars.retain(|key| !MOST_COMMON.contains(key));
    for i in 0..crossover_point {
        let current_key = first_new_individual.keys[i / 3][i % 3];
        first_missing_most_common.retain(|&key| key != current_key);
        first_missing_chars.retain(|&key| key != current_key);
    }

    for i in crossover_point..30 {
        let x = i / 3;
        let y = i % 3;
        if y == 1 && (x > 5 || x < 4) {
            if first_missing_most_common.contains(&second_individual.keys[x][y]) {
                first_new_individual.keys[x][y] = second_individual.keys[x][y];
                first_missing_most_common.retain(|&key| key != second_individual.keys[x][y]);
            }
        } else {
            if first_missing_chars.contains(&second_individual.keys[x][y]) {
                first_new_individual.keys[x][y] = second_individual.keys[x][y];
                first_missing_chars.retain(|&key| key != second_individual.keys[x][y]);
            }
        }
    }

    first_missing_most_common.shuffle(&mut thread_rng());
    first_missing_chars.shuffle(&mut thread_rng());

    for i in crossover_point..30 {
        let x = i / 3;
        let y = i % 3;
        if y == 1 && (x > 5 || x < 4) {
            if first_new_individual.keys[x][y] == '-' {
                first_new_individual.keys[x][y] = first_missing_most_common.pop().unwrap();
            }
        } else {
            if first_new_individual.keys[x][y] == '-' {
                first_new_individual.keys[x][y] = first_missing_chars.pop().unwrap();
            }
        }
    }

    // Fill in missing in second
    let mut second_missing_most_common = MOST_COMMON.to_vec();
    let mut second_missing_chars = CHARS.to_vec();
    second_missing_chars.retain(|key| !MOST_COMMON.contains(key));
    for i in crossover_point..30 {
        let current_key = second_new_individual.keys[i / 3][i % 3];
        second_missing_most_common.retain(|&key| key != current_key);
        second_missing_chars.retain(|&key| key != current_key);
    }

    for i in 0..crossover_point {
        let x = i / 3;
        let y = i % 3;
        if y == 1 && (x > 5 || x < 4) {
            if second_missing_most_common.contains(&first_individual.keys[x][y]) {
                second_new_individual.keys[x][y] = first_individual.keys[x][y];
                second_missing_most_common.retain(|&key| key != first_individual.keys[x][y]);
            }
        } else {
            if second_missing_chars.contains(&first_individual.keys[x][y]) {
                second_new_individual.keys[x][y] = first_individual.keys[x][y];
                second_missing_chars.retain(|&key| key != first_individual.keys[x][y]);
            }
        }
    }

    second_missing_most_common.shuffle(&mut thread_rng());
    second_missing_chars.shuffle(&mut thread_rng());

    for i in 0..crossover_point {
        let x = i / 3;
        let y = i % 3;
        if y == 1 && (x > 5 || x < 4) {
            if second_new_individual.keys[x][y] == '-' {
                second_new_individual.keys[x][y] = second_missing_most_common.pop().unwrap();
            }
        } else {
            if second_new_individual.keys[x][y] == '-' {
                second_new_individual.keys[x][y] = second_missing_chars.pop().unwrap();
            }
        }
    }

    (first_new_individual, second_new_individual)
}

fn mutate(individual: &mut Keyboard) {
    let mut rng = rand::thread_rng();
    for x in 0..10 {
        for y in 0..3 {
            if rng.gen::<f64>() < MUTATION_PROBABILITY {
                if y == 1 && (x > 5 || x < 4) {
                    let mut other_x = rng.gen::<usize>() % 8;
                    if other_x > 3 {
                        other_x += 2;
                    }

                    let tmp = individual.keys[x][1];
                    individual.keys[x][1] = individual.keys[other_x][1];
                    individual.keys[other_x][1] = tmp;
                } else {
                    let (other_x, other_y) = loop {
                        let other_y = rng.gen::<usize>() % 3;
                        let other_x = rng.gen::<usize>() % 10;
                        if other_y != 1 {
                            break (other_x, other_y);
                        }

                        if other_x == 4 || other_x == 5 {
                            break (other_x, other_y);
                        }
                    };

                    let tmp = individual.keys[x][y];
                    individual.keys[x][y] = individual.keys[other_x][other_y];
                    individual.keys[other_x][other_y] = tmp;
                }
            }
        }
    }
}

fn main() {
    let mut population = generate_population();

    for g in 0..GENERATIONS {
        let fitnesses = population
            .iter()
            .map(|individual| evaluate_individual(individual))
            .collect::<Vec<_>>();
        let mut fitnesses_with_index = fitnesses.iter().enumerate().collect::<Vec<_>>();
        fitnesses_with_index.sort_by(|(_, left), (_, right)| {
            left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
        });

        let parent_indices = (0..POPULATION_SIZE)
            .map(|_| tournament_selection(&fitnesses))
            .collect::<Vec<_>>();
        let mut new_population = parent_indices
            .par_chunks(2)
            .flat_map(|parent_indices| {
                let first_individual = &population[parent_indices[0]];
                let second_individual = &population[parent_indices[1]];

                let (first_new, second_new) = cross(first_individual, second_individual);
                vec![first_new, second_new]
            })
            .collect::<Vec<_>>();
        new_population
            .par_iter_mut()
            .for_each(|individual| mutate(individual));

        new_population[0] = population[fitnesses_with_index[0].0].clone();

        println!("best fitness ({}): {}", g, fitnesses_with_index[0].1);

        population = new_population;
    }

    let mut ranked_population = population
        .into_iter()
        .map(|individual| {
            let fitness = evaluate_individual(&individual);
            (individual, fitness)
        })
        .collect::<Vec<_>>();
    ranked_population.sort_by(|(_, left), (_, right)| {
        left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
    });
    ranked_population[0].0.print();
    println!("{}", evaluate_individual(&QWERTY));
    println!("{}", evaluate_individual(&DVORAK));
}
