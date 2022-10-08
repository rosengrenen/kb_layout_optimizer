use lazy_static::lazy_static;
use rand::{seq::SliceRandom, thread_rng, Rng};
use rayon::{
    prelude::{
        IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
    },
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

        unreachable!("{}", key);
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
const COLEMAK: Keyboard = Keyboard {
    keys: [
        ['q', 'a', 'z'],
        ['w', 'r', 'x'],
        ['f', 's', 'c'],
        ['p', 't', 'v'],
        ['g', 'd', 'b'],
        ['j', 'h', 'k'],
        ['l', 'n', 'm'],
        ['u', 'e', ','],
        ['y', 'i', '.'],
        [';', 'o', '?'],
    ],
};
const DVORAK: Keyboard = Keyboard {
    keys: [
        ['?', 'a', ';'],
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
const POPULATION_SIZE: usize = 200;
const GENERATIONS: usize = 1000;
const TOURNAMENT_PROBABILTY: f64 = 0.8;
const TOURNAMENT_SIZE: usize = 2;
const MUTATION_PROBABILITY: f64 = 0.05;
const CROSSOVER_PROBABILITY: f64 = 0.8;

fn generate_individual() -> Keyboard {
    let mut chars = CHARS.to_vec();
    chars.shuffle(&mut thread_rng());
    let mut keyboard = Keyboard::default();

    for i in 0..10 {
        keyboard.keys[i][0] = chars.pop().unwrap();
        keyboard.keys[i][1] = chars.pop().unwrap();
        keyboard.keys[i][2] = chars.pop().unwrap();
    }

    keyboard
}

fn generate_population() -> Vec<Keyboard> {
    let mut keyboards = Vec::with_capacity(POPULATION_SIZE);
    for _ in 0..POPULATION_SIZE {
        keyboards.push(generate_individual());
    }

    keyboards
}

// Lika använding av händer och fingrar för jämt slitage, eller proportioneligt på något vis iallafall.
// Ta hänsyn till musanvänding, typ dominant hand
// Fundera på rimlig input att beräkna kostnaden på, typ vilken text/kod. Sampla github?
// Finger rolling, att använda fingrar brevid för nästa key, behöver kanske mer än en window size på 2 (bigram)
// Vikt/kostnad för att flytta olika fingrar i olika riktningar
// flera lager, men det måste kosta mer att använda tummen pga koordination och stoppar flowet

// params (with weights):
//  * alternating hands
//  * disfavour bottom row
//  * rolls, outer to inner finger mostly, more awkward inner to outer direction
//  * finger strength, i.e. pinky is weaker
//  * hand/finger usage symmetry symmetry
//  * idle time of fingers
//  * physical restrictions of fingers in a hand
//			e.g. one first on top row and the adjacent finger on bottom row on consecutive keys is bad
fn evaluate_individual(individual: &Keyboard, input: &str) -> f64 {
    let mut prev_finger_index = 0;
    let mut fitness = 0.0;
    let same_finger_multiplier = 5.0;
    let top_multipler = 1.0;
    let bottom_multipler = 2.0;
    let calc_distance = |prev_x: isize, prev_y: isize, x: isize, y: isize| {
        let y_delta = (prev_y - y).abs();
        let y_distance: f64 = match y_delta {
            2 => top_multipler + bottom_multipler,
            1 => match prev_y {
                0 => top_multipler,
                1 => match y {
                    0 => top_multipler,
                    2 => bottom_multipler,
                    _ => unreachable!(),
                },
                2 => bottom_multipler,
                _ => unreachable!(),
            },
            0 => 0.0,
            _ => unreachable!(),
        };
        let x_distance = (prev_x - x).abs() as f64;
        (x_distance.powi(2) + y_distance.powi(2)).sqrt()
    };

    for key in input.chars() {
        let finger_index = individual.key_finger(key);
        let (x, y) = individual.key_pos(key);
        let x = x as isize;
        let y = y as isize;

        let start_y = 1;
        let start_x = finger_index as isize;

        if finger_index == prev_finger_index {
            let distance = calc_distance(start_x, start_y, x, y);
            fitness += distance * same_finger_multiplier;
        } else {
            let distance = calc_distance(start_x, start_y, x, y);
            fitness += distance;
        }

        prev_finger_index = finger_index;
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
    if rand::random::<f64>() > CROSSOVER_PROBABILITY {
        return (first_individual.clone(), second_individual.clone());
    }

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
    let mut first_missing_chars = CHARS.to_vec();
    for i in 0..crossover_point {
        let current_key = first_new_individual.keys[i / 3][i % 3];
        first_missing_chars.retain(|&key| key != current_key);
    }

    for i in crossover_point..30 {
        let x = i / 3;
        let y = i % 3;
        if first_missing_chars.contains(&second_individual.keys[x][y]) {
            first_new_individual.keys[x][y] = second_individual.keys[x][y];
            first_missing_chars.retain(|&key| key != second_individual.keys[x][y]);
        }
    }

    first_missing_chars.shuffle(&mut thread_rng());

    for i in crossover_point..30 {
        let x = i / 3;
        let y = i % 3;
        if first_new_individual.keys[x][y] == '-' {
            first_new_individual.keys[x][y] = first_missing_chars.pop().unwrap();
        }
    }

    // Fill in missing in second
    let mut second_missing_chars = CHARS.to_vec();
    for i in crossover_point..30 {
        let current_key = second_new_individual.keys[i / 3][i % 3];
        second_missing_chars.retain(|&key| key != current_key);
    }

    for i in 0..crossover_point {
        let x = i / 3;
        let y = i % 3;
        if second_missing_chars.contains(&first_individual.keys[x][y]) {
            second_new_individual.keys[x][y] = first_individual.keys[x][y];
            second_missing_chars.retain(|&key| key != first_individual.keys[x][y]);
        }
    }

    second_missing_chars.shuffle(&mut thread_rng());

    for i in 0..crossover_point {
        let x = i / 3;
        let y = i % 3;
        if second_new_individual.keys[x][y] == '-' {
            second_new_individual.keys[x][y] = second_missing_chars.pop().unwrap();
        }
    }

    (first_new_individual, second_new_individual)
}

// rosens tagning: gå igenom alla keys och kolla om det ska mutera, lägg till deras keys i en pool, sätt alla nya keys som inte
// muterar till det de var innan, och slumpa ut från poolen till de som ska mutera
// dinos tagning: slumpa ordningen av index, gå igenom den i ordning, om en key finns i poolen (av alla keys) och inte ska mutera,
// tar den den, annars tar den en slumpad
fn mutate(individual: &mut Keyboard) {
    let mut rng = rand::thread_rng();
    let mut keys = Vec::new();
    let mut key_positions = Vec::new();
    for x in 0..10 {
        for y in 0..3 {
            if rng.gen::<f64>() < MUTATION_PROBABILITY {
                keys.push(individual.keys[x][y]);
                key_positions.push((x, y));
            }
        }
    }

    keys.shuffle(&mut rng);
    for x in 0..10 {
        for y in 0..3 {
            if key_positions.contains(&(x, y)) {
                individual.keys[x][y] = keys.pop().unwrap();
            }
        }
    }
}

fn main() {
    let input = include_str!("../data/eng-uk_web_2202_300K/eng-uk_web_2002_300K-sentences.txt")
        .lines()
        .step_by(50)
        .map(|line| {
            let (_, content) = line.split_once("\t").unwrap();
            content
                .chars()
                .filter(|c| CHARS.contains(c))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("");

    println!("qwerty: {}", evaluate_individual(&QWERTY, &input));
    println!("colemak: {}", evaluate_individual(&COLEMAK, &input));
    println!("dvorak: {}", evaluate_individual(&DVORAK, &input));

    let mut population = generate_population();

    for g in 0..GENERATIONS {
        let fitnesses = population
            .par_iter()
            .map(|individual| evaluate_individual(individual, &input))
            .collect::<Vec<_>>();
        let mut fitnesses_with_index = fitnesses.iter().enumerate().collect::<Vec<_>>();
        fitnesses_with_index.sort_by(|(_, left), (_, right)| {
            left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
        });

        println!(
            "[{}] best: {:.1}, avg: {:.1}",
            g,
            fitnesses_with_index[0].1,
            fitnesses.iter().sum::<f64>() / POPULATION_SIZE as f64
        );
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

        population = new_population;
    }

    let mut ranked_population = population
        .into_par_iter()
        .map(|individual| {
            let fitness = evaluate_individual(&individual, &input);
            (individual, fitness)
        })
        .collect::<Vec<_>>();
    ranked_population.sort_by(|(_, left), (_, right)| {
        left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
    });
    ranked_population[0].0.print();
}
