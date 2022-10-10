use std::collections::HashMap;

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

    fn print_freq(&self, letter_freq: &HashMap<char, f64>) {
        let mut left_hand_usage = 0.0;
        for x in 0..5 {
            for y in 0..3 {
                left_hand_usage += letter_freq.get(&self.keys[x][y]).unwrap();
            }
        }

        println!("Hand usage: {} {}", left_hand_usage, 1.0 - left_hand_usage);
        for y in 0..3 {
            for x in 0..10 {
                print!("{:.04} ", letter_freq.get(&self.keys[x][y]).unwrap());
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
const MOST_COMMON_CHARS: [char; 8] = ['e', 't', 'a', 'o', 'i', 'n', 's', 'r'];
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
const MIRYOKU: Keyboard = Keyboard {
    keys: [
        ['q', 'a', 'z'],
        ['w', 'r', 'x'],
        ['f', 's', 'c'],
        ['p', 't', 'd'],
        ['b', 'g', 'v'],
        ['j', 'm', 'k'],
        ['l', 'n', 'h'],
        ['u', 'e', ','],
        ['y', 'i', '.'],
        [';', 'o', '?'],
    ],
};
const CANDIDATE_1: Keyboard = Keyboard {
    keys: [
        ['p', 'o', 'z'],
        ['h', 'a', 'k'],
        ['f', 'e', ','],
        ['y', 'i', ';'],
        ['b', 'u', '.'],
        ['g', 'm', '?'],
        ['w', 's', 'q'],
        ['d', 't', 'x'],
        ['l', 'n', 'v'],
        ['c', 'r', 'j'],
    ],
};
// p h f y b g w d l c
// o a e i u m s t n r
// z k , ; . ? q x v j

// Settings
const POPULATION_SIZE: usize = 200;
const GENERATIONS: usize = 500;
const TOURNAMENT_PROBABILTY: f64 = 0.8;
const TOURNAMENT_SIZE: usize = 2;
const MUTATION_PROBABILITY: f64 = 1.0 / 22.0;
const MUTATION_PROBABILITY_COMMON: f64 = 1.0 / 8.0;
const CROSSOVER_PROBABILITY: f64 = 0.8;

fn generate_individual() -> Keyboard {
    let mut most_common_chars = MOST_COMMON_CHARS.to_vec();
    most_common_chars.shuffle(&mut thread_rng());
    let mut chars = CHARS.to_vec();
    chars.retain(|key| !most_common_chars.contains(key));
    chars.shuffle(&mut thread_rng());
    let mut keyboard = Keyboard::default();

    for i in 0..10 {
        keyboard.keys[i][0] = chars.pop().unwrap();
        keyboard.keys[i][2] = chars.pop().unwrap();
        if i < 4 || i > 5 {
            keyboard.keys[i][1] = most_common_chars.pop().unwrap();
        } else {
            keyboard.keys[i][1] = chars.pop().unwrap();
        }
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
//  * dont use the same finger for two keys in row
const SAME_FINGER_PENALTY: f64 = 2.0;

//  * alternating hands
const SAME_HAND_PENALTY: f64 = 2.0;

//  * disfavour bottom row
const BOTTOM_ROW_PENALTY: f64 = 5.0;

//  * rolls, outer to inner finger mostly, more awkward inner to outer direction

//  * finger strength, i.e. pinky is weaker
//  * hand/finger usage symmetry symmetry
const FINGER_TARGET_USAGE: [f64; 10] = [0.1, 0.135, 0.135, 0.13, 0.0, 0.0, 0.13, 0.135, 0.135, 0.1];

//  * idle time of fingers
//  * physical restrictions of fingers in a hand
//			e.g. one first on top row and the adjacent finger on bottom row on consecutive keys is bad
fn evaluate_individual(individual: &Keyboard, input: &str) -> f64 {
    let mut prev_finger_index = 0;
    let mut fitness = 0.0;
    let calc_distance = |prev_x: isize, prev_y: isize, x: isize, y: isize| {
        let top = prev_y == 0 || y == 0;
        let bottom = prev_y == 2 || y == 2;
        let x_distance = (prev_x - x).abs() as f64;
        let y_distance = match (top, bottom) {
            (true, true) => 1.0 + BOTTOM_ROW_PENALTY,
            (true, false) => 1.0,
            (false, true) => BOTTOM_ROW_PENALTY,
            (false, false) => 0.0,
        };
        (x_distance.powi(2) + y_distance.powi(2)).sqrt()
    };

    let mut finger_positions = [
        (0, 1),
        (1, 1),
        (2, 1),
        (3, 1),
        (4, 1),
        (5, 1),
        (6, 1),
        (7, 1),
        (8, 1),
        (9, 1),
    ];
    let mut finger_usage = [0; 10];
    let mut same_finger_count = 0;
    let mut same_hand_no_roll_count = 0;
    let mut prev_key = '-';
    for key in input.chars() {
        let finger_idx = individual.key_finger(key);
        let (x, y) = individual.key_pos(key);
        let x = x as isize;
        let y = y as isize;

        let (start_x, start_y) = finger_positions[finger_idx];

        fitness += calc_distance(start_x, start_y, x, y);
        if finger_idx == prev_finger_index && prev_key != key {
            same_finger_count += 1;
        }

        let same_hand = match (finger_idx, prev_finger_index) {
            (0..=4, 0..=4) => true,
            (5..=9, 5..=9) => true,
            _ => false,
        };

        let normalize_finger_idx = |finger_idx: usize| {
            if finger_idx > 4 {
                9 - finger_idx
            } else {
                finger_idx
            }
        };
        let normalized_finger_idx = normalize_finger_idx(finger_idx);
        let prev_normalized_finger_idx = normalize_finger_idx(prev_finger_index);
        if same_hand && normalized_finger_idx < prev_normalized_finger_idx {
            same_hand_no_roll_count += 1;
        }

        finger_positions[finger_idx] = (x, y);
        finger_usage[finger_idx] += 1;
        prev_finger_index = finger_idx;
        prev_key = key;
    }

    fitness += same_finger_count as f64 * SAME_FINGER_PENALTY;
    fitness += same_hand_no_roll_count as f64 * SAME_HAND_PENALTY;

    let total_usage = finger_usage.iter().sum::<i32>() as f64;
    fitness *= 1.0
        + finger_usage
            .into_iter()
            .enumerate()
            .map(|(finger_idx, usage)| {
                (1.0 + (FINGER_TARGET_USAGE[finger_idx] - usage as f64 / total_usage).abs()).powi(3)
                    - 1.0
            })
            .sum::<f64>();

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
    let mut first_missing_common_chars = MOST_COMMON_CHARS.to_vec();
    let mut first_missing_chars = CHARS.to_vec();
    first_missing_chars.retain(|key| !MOST_COMMON_CHARS.contains(key));
    for i in 0..crossover_point {
        let current_key = first_new_individual.keys[i / 3][i % 3];
        first_missing_chars.retain(|&key| key != current_key);
        first_missing_common_chars.retain(|&key| key != current_key);
    }

    for i in crossover_point..30 {
        let x = i / 3;
        let y = i % 3;
        if first_missing_chars.contains(&second_individual.keys[x][y]) {
            first_new_individual.keys[x][y] = second_individual.keys[x][y];
            first_missing_chars.retain(|&key| key != second_individual.keys[x][y]);
        } else if first_missing_common_chars.contains(&second_individual.keys[x][y]) {
            first_new_individual.keys[x][y] = second_individual.keys[x][y];
            first_missing_common_chars.retain(|&key| key != second_individual.keys[x][y]);
        }
    }

    first_missing_common_chars.shuffle(&mut thread_rng());
    first_missing_chars.shuffle(&mut thread_rng());

    for i in crossover_point..30 {
        let x = i / 3;
        let y = i % 3;
        if first_new_individual.keys[x][y] == '-' {
            if y == 1 && (x < 4 || x > 5) {
                first_new_individual.keys[x][y] = first_missing_common_chars.pop().unwrap();
            } else {
                first_new_individual.keys[x][y] = first_missing_chars.pop().unwrap();
            }
        }
    }

    // Fill in missing in second
    let mut second_missing_common_chars = MOST_COMMON_CHARS.to_vec();
    let mut second_missing_chars = CHARS.to_vec();
    second_missing_chars.retain(|key| !MOST_COMMON_CHARS.contains(key));
    for i in crossover_point..30 {
        let current_key = second_new_individual.keys[i / 3][i % 3];
        second_missing_chars.retain(|&key| key != current_key);
        second_missing_common_chars.retain(|&key| key != current_key);
    }

    for i in 0..crossover_point {
        let x = i / 3;
        let y = i % 3;
        if second_missing_chars.contains(&first_individual.keys[x][y]) {
            second_new_individual.keys[x][y] = first_individual.keys[x][y];
            second_missing_chars.retain(|&key| key != first_individual.keys[x][y]);
        } else if second_missing_common_chars.contains(&first_individual.keys[x][y]) {
            second_new_individual.keys[x][y] = first_individual.keys[x][y];
            second_missing_common_chars.retain(|&key| key != first_individual.keys[x][y]);
        }
    }

    second_missing_common_chars.shuffle(&mut thread_rng());
    second_missing_chars.shuffle(&mut thread_rng());

    for i in 0..crossover_point {
        let x = i / 3;
        let y = i % 3;
        if second_new_individual.keys[x][y] == '-' {
            if y == 1 && (x < 4 || x > 5) {
                second_new_individual.keys[x][y] = second_missing_common_chars.pop().unwrap();
            } else {
                second_new_individual.keys[x][y] = second_missing_chars.pop().unwrap();
            }
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
    let mut common_keys = Vec::new();
    let mut common_key_positions = Vec::new();
    for x in 0..10 {
        for y in 0..3 {
            if y == 1 && (x < 4 || x > 5) {
                if rng.gen::<f64>() < MUTATION_PROBABILITY_COMMON {
                    common_keys.push(individual.keys[x][y]);
                    common_key_positions.push((x, y));
                }
            } else {
                if rng.gen::<f64>() < MUTATION_PROBABILITY {
                    keys.push(individual.keys[x][y]);
                    key_positions.push((x, y));
                }
            }
        }
    }

    keys.shuffle(&mut rng);
    common_keys.shuffle(&mut rng);
    for x in 0..10 {
        for y in 0..3 {
            if key_positions.contains(&(x, y)) {
                individual.keys[x][y] = keys.pop().unwrap();
            } else if common_key_positions.contains(&(x, y)) {
                individual.keys[x][y] = common_keys.pop().unwrap();
            }
        }
    }
}

fn main() {
    let mut raw_input =
        include_str!("../data/eng-uk_web_2202_300K/eng-uk_web_2002_300K-sentences.txt")
            .lines()
            .collect::<Vec<_>>();
    raw_input.shuffle(&mut rand::thread_rng());
    let input = raw_input
        .iter()
        .step_by(50)
        .map(|line| {
            let (_, content) = line.split_once("\t").unwrap();
            content
                .to_lowercase()
                .chars()
                .filter(|c| c.is_ascii_alphabetic())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("");
    let mut letter_freq: HashMap<char, f64> = HashMap::new();
    let mut bigrams: HashMap<String, f64> = HashMap::new();
    let mut prev_c = None;
    for line in raw_input.iter() {
        for c in line.chars() {
            let c = c.to_ascii_lowercase();
            if !CHARS.contains(&c) {
                continue;
            }

            *letter_freq.entry(c).or_default() += 1.0;

            if let Some(prev_c) = prev_c {
                let bigram = format!("{}{}", prev_c, c);
                *bigrams.entry(bigram).or_default() += 1.0;
            }

            prev_c = Some(c);
        }
    }

    let len: f64 = letter_freq.values().sum();
    for f in letter_freq.values_mut() {
        *f /= len;
    }

    let len: f64 = bigrams.values().sum();
    for b in bigrams.values_mut() {
        *b /= len;
    }

    println!("qwerty: {}", evaluate_individual(&QWERTY, &input));
    QWERTY.print_freq(&letter_freq);
    println!("colemak: {}", evaluate_individual(&COLEMAK, &input));
    COLEMAK.print_freq(&letter_freq);
    println!("dvorak: {}", evaluate_individual(&DVORAK, &input));
    DVORAK.print_freq(&letter_freq);
    println!("miryoku: {}", evaluate_individual(&MIRYOKU, &input));
    MIRYOKU.print_freq(&letter_freq);
    println!("candidate 1: {}", evaluate_individual(&CANDIDATE_1, &input));
    CANDIDATE_1.print_freq(&letter_freq);

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
    ranked_population[0].0.print_freq(&letter_freq);
}
