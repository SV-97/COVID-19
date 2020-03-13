#![allow(dead_code)]
use rand::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::channel;

static AGE_X: [f32; 11] = [
    0., 0.00944196, 0.05640964, 0.13170318, 0.1596659, 0.19101804, 0.2360489, 0.42779325,
    0.71710447, 0.78356131, 1.,
];

static AGE_Y: [f32; 11] = [0., 1., 5., 14., 17., 20., 24., 39., 59., 64., 100.];

fn get_age(x: f32) -> usize {
    for i in 0..9 {
        if AGE_X[i] <= x && x < AGE_X[i + 1] {
            let val =
                (AGE_Y[i + 1] - AGE_Y[i]) / (AGE_X[i + 1] - AGE_X[i]) * (x - AGE_X[i]) + AGE_Y[i]; // linear interpolation
            return val as usize;
        }
    }
    return AGE_Y[10] as usize;
}

pub fn age_distribution(group: &[Person]) -> HashMap<usize, usize> {
    let mut res: HashMap<usize, usize> = HashMap::new();
    for person in group {
        let age = 10 * (person.age / 10);
        let entry = res.entry(age).or_insert(0);
        *entry += 1;
    }
    res
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Person {
    age: usize,
    days_since_infection: usize,
}

impl Person {
    pub fn new() -> Self {
        Self {
            age: get_age(rand::random()),
            days_since_infection: 0,
        }
    }

    fn cure_function(&self) -> f32 {
        1. / (self.age as f32 / 10.) * (1. - (self.days_since_infection as f32 / -10.).exp())
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
            * match self.age {
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
        0.28 * match self.age {
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
    pub simulated_days: usize,
    pub infected_people: Vec<Person>,
    pub dead_people: Vec<Person>,
    pub cured_people: Vec<Person>,
    pub current_population: Vec<Person>,
    pub running_death_tolls: Vec<usize>,
    pub infections_over_time: Vec<usize>,
}

impl Simulation {
    pub fn new(initial_population: Vec<Person>) -> Self {
        Self {
            simulated_days: 0,
            infected_people: vec![],
            dead_people: vec![],
            cured_people: vec![],
            current_population: initial_population,
            running_death_tolls: vec![],
            infections_over_time: vec![],
        }
    }

    fn simulate_day(&mut self) {
        let mut rng = rand::thread_rng();
        let old_population = {
            let mut new_population = Vec::with_capacity(self.current_population.len());
            std::mem::swap(&mut self.current_population, &mut new_population);
            new_population
        };
        for mut person in old_population {
            person.days_since_infection += 1;
            match person {
                person if person.dies() => self.dead_people.push(person),
                person if person.gets_cured() => self.cured_people.push(person),
                person => {
                    for _ in 0..Person::number_of_people_met(person.days_since_infection) {
                        let p = Person::new();
                        if rng.gen_range(0., 1.) < p.infection_chance() {
                            self.current_population.push(p);
                            self.infected_people.push(p);
                        }
                    }
                    self.current_population.push(person.clone());
                }
            }
        }
        self.infections_over_time.push(self.infected_people.len());
        self.running_death_tolls.push(self.dead_people.len());
        self.simulated_days += 1;
    }

    pub fn run_simulation(&mut self, cancellation_pred: impl Fn(&Self) -> bool) -> &mut Self {
        while !cancellation_pred(self) {
            self.simulate_day();
        }
        self
    }
}

#[derive(Debug)]
pub struct ParallelSimulation {
    number_of_threads: usize,
    pub simulated_days: usize,
    pub infected_people: Vec<Person>,
    pub dead_people: Vec<Person>,
    pub cured_people: Vec<Person>,
    pub current_population: Vec<Person>,
    pub running_death_tolls: Vec<usize>,
    pub infections_over_time: Vec<usize>,
}

enum PersonMessage {
    Dead(Person),
    Cured(Person),
}

impl ParallelSimulation {
    pub fn new(number_of_threads: usize, initial_population: Vec<Person>) -> Self {
        Self {
            number_of_threads,
            simulated_days: 0,
            infected_people: vec![],
            dead_people: vec![],
            cured_people: vec![],
            current_population: initial_population,
            running_death_tolls: vec![],
            infections_over_time: vec![],
        }
    }

    fn simulate_day(&mut self) {
        let mut rng = rand::thread_rng();
        let old_population = {
            let mut new_population = Vec::with_capacity(self.current_population.len());
            std::mem::swap(&mut self.current_population, &mut new_population);
            new_population
        };
        for mut person in old_population {
            person.days_since_infection += 1;
            match person {
                person if person.dies() => self.dead_people.push(person),
                person if person.gets_cured() => self.cured_people.push(person),
                person => {
                    for _ in 0..Person::number_of_people_met(person.days_since_infection) {
                        let p = Person::new();
                        if rng.gen_range(0., 1.) < p.infection_chance() {
                            self.current_population.push(p);
                            self.infected_people.push(p);
                        }
                    }
                    self.current_population.push(person.clone());
                }
            }
        }
        self.infections_over_time.push(self.infected_people.len());
        self.running_death_tolls.push(self.dead_people.len());
        self.simulated_days += 1;
    }

    fn simulate_day_parallel(&mut self) {
        let old_population = {
            let mut new_population = Vec::with_capacity(self.current_population.len());
            std::mem::swap(&mut self.current_population, &mut new_population);
            new_population
        };
        let per_thread = old_population.len() / self.number_of_threads;
        let mut threads = Vec::with_capacity(self.number_of_threads);
        let (sender, receiver) = channel();
        for chunk in old_population.chunks(per_thread) {
            if chunk.len() == 0 {
                continue;
            }
            let channel = sender.clone();
            let chunk = chunk.to_owned();
            threads.push(std::thread::spawn(move || {
                let channel = channel;
                let mut rng = rand::thread_rng();
                let mut remainer_chunk = Vec::with_capacity(chunk.len());
                let mut newly_infected_chunk = Vec::with_capacity(chunk.len());
                for mut person in chunk {
                    person.days_since_infection += 1;
                    match person {
                        person if person.dies() => {
                            channel.send(PersonMessage::Dead(person.clone())).unwrap()
                        }
                        person if person.gets_cured() => {
                            channel.send(PersonMessage::Cured(person.clone())).unwrap()
                        }
                        person => {
                            for _ in 0..Person::number_of_people_met(person.days_since_infection) {
                                let p = Person::new();
                                if rng.gen_range(0., 1.) < p.infection_chance() {
                                    newly_infected_chunk.push(p);
                                }
                            }
                            remainer_chunk.push(person.clone());
                        }
                    }
                }
                drop(channel);
                (remainer_chunk, newly_infected_chunk)
            }));
        }
        drop(sender);
        for message in receiver {
            match message {
                PersonMessage::Dead(person) => self.dead_people.push(person),
                PersonMessage::Cured(person) => self.cured_people.push(person),
            }
        }
        for thread in threads {
            let (remainers, newly_infected) = thread.join().unwrap();
            self.current_population.extend(remainers);
            self.current_population.extend(newly_infected.clone());
            self.infected_people.extend(newly_infected);
        }
        self.infections_over_time.push(self.infected_people.len());
        self.running_death_tolls.push(self.dead_people.len());
        self.simulated_days += 1;
    }

    pub fn run_simulation(&mut self, cancellation_pred: impl Fn(&Self) -> bool) -> &mut Self {
        while !cancellation_pred(self) {
            if self.current_population.len() < 100 * self.number_of_threads {
                self.simulate_day();
            } else {
                self.simulate_day_parallel();
            }
        }
        self
    }
}
