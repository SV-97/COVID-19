#![feature(clamp)]
#![feature(bindings_after_at)]
mod sim;

use sim::{age_distribution, Person, Simulation};

fn main() {
    let mut sim = Simulation::new(8, vec![Person::new_in_simulation()]);
    sim.run_simulation(|sim| sim.simulated_days >= 365 || sim.people.len() > 80_000_000);
    dbg!(age_distribution(sim.people.iter().filter(|p| p.is_dead())));
    dbg!(age_distribution(sim.people.iter().filter(|p| p.is_cured())));
    dbg!(age_distribution(
        sim.people.iter().filter(|p| p.is_in_simulation())
    ));
    dbg!(sim.simulated_days);
    dbg!(sim.people.len());
}
