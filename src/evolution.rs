use super::population::Population;
use super::operations::{
    mutate_some_agents,
    mate_some_agents,
    cull_lowest_agents,
    mate_alpha_agents
};
use std::thread;
use rand::{
    distributions::{Distribution, Standard}
};
use std::hash::Hash;
use super::agent::Agent;

pub fn population_from_multilevel_sub_populations<Gene, IndexFunction, Data>(
    levels: u32,
    sub_populations_per_level: usize,
    data: Data,
    number_of_genes: usize,
    initial_population_size: usize,
    iterations_on_each_population: usize,
    get_score_index: &'static IndexFunction) -> Population<Gene> 
where Gene: Clone + PartialEq + Hash + Send + 'static, Standard: Distribution<Gene>,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
Data: Clone + Send + 'static
    {
    let number_of_initial_populations = sub_populations_per_level.pow(levels);
    let mut populations = Vec::new();
    for _ in 0..number_of_initial_populations {
        populations.push(run_iterations(create_population(initial_population_size, &data, number_of_genes, get_score_index), iterations_on_each_population, &data, false, get_score_index));
    }

    populations_from_existing_multillevel(populations, levels, sub_populations_per_level, &data, iterations_on_each_population, get_score_index)
}

pub fn threaded_population_from_multilevel_sub_populations<Gene, IndexFunction, Data>(
    levels: u32,
    sub_populations_per_level: usize,
    data: &Data,
    number_of_genes: usize,
    initial_population_size: usize,
    iterations_on_each_population: usize,
    get_score_index: &'static IndexFunction) -> Population<Gene> 
where Gene: Clone + PartialEq + Send + Hash + 'static, Standard: Distribution<Gene>,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
Data: Clone + Send + 'static
    {
    let mut populations = Vec::new();
    let mut handles = Vec::new();
    for _ in 0..sub_populations_per_level {
        let data_copy = data.clone();
        handles.push(thread::spawn(move || population_from_multilevel_sub_populations(levels - 1, sub_populations_per_level, data_copy, number_of_genes, initial_population_size, iterations_on_each_population, get_score_index)));
    }

    for handle in handles {
        populations.push(handle.join().unwrap());
    }

    populations_from_existing_multillevel(populations, 1, sub_populations_per_level, data, iterations_on_each_population, get_score_index)
}

fn populations_from_existing_multillevel<Gene, IndexFunction, Data>(
    mut populations: Vec<Population<Gene>>,
    levels: u32,
    sub_populations_per_level: usize,
    data: &Data,
    iterations_on_each_population: usize,
    get_score_index: &'static IndexFunction) -> Population<Gene>
where Gene: Clone + PartialEq + Hash + Send + 'static, Standard: Distribution<Gene>,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
Data: Clone + Send + 'static
    {                 
    for level in (0..levels).rev() {
        let number_of_new_populations = sub_populations_per_level.pow(level);
        let mut new_populations = Vec::new();
        for _ in 0..number_of_new_populations {
            let mut population = Population::new_empty(false);
            for _ in 0..sub_populations_per_level {
                let sub_population = populations.pop().unwrap();
                for (score, agent) in sub_population.get_agents() {
                    population.insert(*score, agent.clone());
                }
            }
            new_populations.push(cull_lowest_agents(run_iterations(population, iterations_on_each_population, data, false, get_score_index), 0.75));
        }

        populations = new_populations;
    }

    populations.pop().unwrap()
}

fn create_population<Gene, IndexFunction, Data>(
    start_size: usize,
    data: &Data,
    number_of_genes: usize,
    get_score_index: &'static IndexFunction) -> Population<Gene>
where Gene: Clone + PartialEq + Hash, Standard: Distribution<Gene>,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
Data: Clone
    {
    let mut population = Population::new_empty(false);
    for _ in 0..start_size {
        let agent = Agent::new(number_of_genes);
        if population.will_accept(&agent) {
            let mut score = get_score_index(&agent, &data);

            loop {
                if score == 0 {
                    break;
                }
                if population.contains_score(score) {
                    score -= 1;
                } else {
                    break;
                }
            }

            population.insert(score, agent);
        }
    }

    population
}

fn run_iterations<Gene, IndexFunction, Data>(
    mut population: Population<Gene>,
    iterations: usize,
    data: &Data,
    print_progress: bool, 
    get_score_index: &'static IndexFunction) -> Population<Gene>
where Gene: Clone + PartialEq + Hash + Send + 'static, Standard: Distribution<Gene>,
IndexFunction: Send + Sync + Fn(&Agent<Gene>, &Data) -> isize + 'static,
Data: Clone + Send + 'static
    {
    for x in 0..iterations {
        population = mutate_some_agents(population, 0.1, data, get_score_index, 1);
        population = mate_alpha_agents(population, 0.2, data, get_score_index, 1, 2500);
        population = mate_some_agents(population, 0.5, data, get_score_index, 1, 1000);
        population = cull_lowest_agents(population, 0.02);

        if print_progress && x % 10 == 0 {
            println!("-- Iteration {} --", x);
            println!("Population: {}", population.len());
            let agents = population.get_agents();
            let (top_score, _) = agents.iter().rev().next().unwrap();
            println!("Top score: {}", top_score);
            println!("------------------");
        }
    }

    population
}