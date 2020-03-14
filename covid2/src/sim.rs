#![allow(dead_code)]
extern crate rand;
use rand::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::thread;

static AGE_X: [f32; 11] = [
    0., 0.00944196, 0.05640964, 0.13170318, 0.1596659, 0.19101804, 0.2360489, 0.42779325,
    0.71710447, 0.78356131, 1.,
];

static AGE_Y: [f32; 11] = [0., 1., 5., 14., 17., 20., 24., 39., 59., 64., 100.];

fn get_age(x: f32) -> u32 {
    for i in 0..9 {
        if AGE_X[i] <= x && x < AGE_X[i + 1] {
            let val =
                (AGE_Y[i + 1] - AGE_Y[i]) / (AGE_X[i + 1] - AGE_X[i]) * (x - AGE_X[i]) + AGE_Y[i]; // linear interpolation
            return val as u32;
        }
    }
    return AGE_Y[10] as u32;
}

pub fn age_distribution<'a, I: IntoIterator<Item = &'a Person>>(group: I) -> HashMap<u32, u32> {
    let mut res: HashMap<u32, u32> = HashMap::new();
    for person in group {
        let age = 10 * (person.age() / 10);
        let entry = res.entry(age).or_insert(0);
        *entry += 1;
    }
    res
}

#[derive(Copy, Clone, Debug)]
pub enum Person {
    InSimulation { age: u32, days_since_infection: u32 },
    Dead { age: u32 },
    Cured { age: u32 },
}

impl Person {
    pub fn new_in_simulation() -> Self {
        Person::InSimulation {
            age: get_age(rand::random()),
            days_since_infection: 0,
        }
    }

    pub fn is_in_simulation(&self) -> bool {
        if let Person::InSimulation { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn is_dead(&self) -> bool {
        if let Person::Dead { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn is_cured(&self) -> bool {
        if let Person::Cured { .. } = self {
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn age(self) -> u32 {
        match self {
            Person::InSimulation { age, .. } => age,
            Person::Dead { age, .. } => age,
            Person::Cured { age, .. } => age,
        }
    }

    #[inline]
    pub fn days_since_infection(self) -> u32 {
        match self {
            Person::InSimulation {
                days_since_infection,
                ..
            } => days_since_infection,
            _ => panic!(),
        }
    }

    fn cure_function(&self) -> f32 {
        1. / (self.age() as f32 / 10.) * (1. - (self.days_since_infection() as f32 / -10.).exp())
    }

    fn number_of_people_met(day: usize) -> usize {
        match day {
            x if x <= 7 => 4,
            x if x <= 21 => 2,
            _ => 2,
        }
    }

    fn death_chance(&self) -> f32 {
        0.065
            * match self.age() {
                age if age <= 9 => 0.,
                age if age <= 39 => 0.002,
                age if age <= 49 => 0.004,
                age if age <= 59 => 0.013,
                age if age <= 69 => 0.036,
                age if age <= 79 => 0.08,
                _ => 0.148,
            }
    }

    fn infection_chance(&self) -> f32 {
        0.28 * match self.age() {
            age if age <= 9 => 0.05,
            age if age <= 19 => 0.1,
            age if age <= 39 => 0.2,
            _ => 0.4,
        }
    }

    fn cure_chance(&self) -> f32 {
        self.cure_function().clamp(0., 1.)
    }

    fn gets_cured(&self) -> bool {
        rand::random::<f32>() < self.cure_chance()
    }

    fn dies(&self) -> bool {
        rand::random::<f32>() < self.death_chance()
    }
}

#[derive(Debug)]
pub struct Simulation {
    number_of_threads: usize,
    pub simulated_days: u32,
    pub people: Vec<Person>,
}

impl Simulation {
    pub fn new(number_of_threads: usize, initial_population: Vec<Person>) -> Self {
        Self {
            number_of_threads,
            simulated_days: 0,
            people: initial_population,
        }
    }

    fn simulate_day(&mut self) {
        let mut rng = rand::thread_rng();

        let mut new_population = Vec::with_capacity(self.people.len());

        for person in std::mem::take(&mut self.people) {
            match person {
                person @ Person::Dead { .. } => new_population.push(person),
                person @ Person::Cured { .. } => new_population.push(person),
                person
                @
                Person::InSimulation {
                    age,
                    days_since_infection,
                } => {
                    if person.dies() {
                        new_population.push(Person::Dead { age });
                    } else if person.gets_cured() {
                        new_population.push(Person::Cured { age });
                    } else {
                        for _ in 0..Person::number_of_people_met(days_since_infection as usize) {
                            let p = Person::new_in_simulation();
                            if rng.gen_range(0., 1.) < p.infection_chance() {
                                new_population.push(p);
                            }
                        }
                        new_population.push(Person::InSimulation {
                            age,
                            days_since_infection: days_since_infection + 1,
                        });
                    }
                }
            }
        }
        self.people = new_population;
        self.simulated_days += 1;
    }

    fn simulate_day_parallel(&mut self) {
        let mut threads = Vec::with_capacity(self.number_of_threads);
        let per_thread = self.people.len() / self.number_of_threads;
        let old_population: Vec<Person> = std::mem::take(&mut self.people);
        for chunk in old_population.chunks(per_thread) {
            let chunk = chunk.to_owned();
            threads.push(thread::spawn(move || {
                let mut rng = rand::thread_rng();
                let mut new_population = Vec::with_capacity(chunk.len());
                for person in chunk {
                    match person.clone() {
                        person @ Person::Dead { .. } => new_population.push(person),
                        person @ Person::Cured { .. } => new_population.push(person),
                        person
                        @
                        Person::InSimulation {
                            age,
                            days_since_infection,
                        } => {
                            if person.gets_cured() {
                                new_population.push(Person::Cured { age });
                            } else if person.dies() {
                                new_population.push(Person::Dead { age });
                            } else {
                                for _ in
                                    0..Person::number_of_people_met(days_since_infection as usize)
                                {
                                    let p = Person::new_in_simulation();
                                    if rng.gen_range(0., 1.) < p.infection_chance() {
                                        new_population.push(p);
                                    }
                                }
                                new_population.push(Person::InSimulation {
                                    age,
                                    days_since_infection: days_since_infection + 1,
                                });
                            }
                        }
                    }
                }
                new_population
            }));
        }

        for thread in threads {
            self.people.extend(thread.join().unwrap());
        }

        self.simulated_days += 1;
    }

    pub fn run_simulation(&mut self, cancellation_pred: impl Fn(&Self) -> bool) -> &mut Self {
        while !cancellation_pred(self) {
            if self.people.len() / self.number_of_threads > 1000 {
                self.simulate_day_parallel();
            } else {
                self.simulate_day();
            }
        }
        self
    }
}
