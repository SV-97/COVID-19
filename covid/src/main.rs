#![feature(clamp)]
extern crate rand;
mod sim;

use sim::{age_distribution, ParallelSimulation, Person, Simulation};
fn main() {
    //let mut sim = Simulation::new(vec![Person::new()]);
    let mut sim = ParallelSimulation::new(8, vec![Person::new()]);
    sim.run_simulation(|sim| {
        sim.simulated_days >= 365 || sim.current_population.len() > 10_000_000
    });
    dbg!(sim.running_death_tolls);
    dbg!(sim.infections_over_time);
    dbg!(sim.simulated_days);
    dbg!(sim.current_population.len());
    dbg!(age_distribution(&sim.current_population));
}
