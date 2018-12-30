use super::agent::Agent;
use std::collections::{BTreeMap, HashSet};
use std::hash::Hash;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};


pub struct Population <Gene> where Gene: Clone {
    agents: BTreeMap<isize, Agent<Gene>>,
    register: HashSet<u64>,
    unique_agents: bool,

}

impl <Gene> Population <Gene>
where 
Standard: Distribution<Gene>,
Gene: Clone + PartialEq + Hash
{

    pub fn new_empty(unique: bool) -> Self {
        Self {
            agents: BTreeMap::new(),
            register: HashSet::new(),
            unique_agents: unique
        }
    }

    pub fn new<Data, IndexFunction>(
        start_size: usize,
        number_of_genes: usize,
        unique: bool,
        data: &Data,
        get_score_index: &'static IndexFunction
        ) -> Population<Gene> 
        where IndexFunction: Fn(&Agent<Gene>, &Data) -> isize 
        {

            let mut population = Population::new_empty(unique);
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

    pub fn set_agents(&mut self, agents: BTreeMap<isize, Agent<Gene>>) {
        for (score, agent) in agents {
            self.insert(score, agent);
        }
    }

    pub fn insert(&mut self, score: isize, agent: Agent<Gene>) {
        if self.unique_agents {
            if self.register.contains(&agent.get_hash()) {
                return;
            }
            self.register.insert(agent.get_hash());
        }
        self.agents.insert(score, agent);
    }

    pub fn remove(&mut self, score: isize) -> Option<Agent<Gene>> {
        let agent = self.agents.remove(&score);
        if self.unique_agents && agent.is_some() {
            self.register.remove(&agent.clone().unwrap().get_hash());
        }
        agent
    }

    pub fn get(&self, score: isize) -> Option<&Agent<Gene>> {
        self.agents.get(&score)
    }

    pub fn get_agents(&self) -> &BTreeMap<isize, Agent<Gene>> {
        &self.agents
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn cull_all_below(&mut self, score: isize) {
        self.agents = self.agents.split_off(&score);
        if self.unique_agents {
            self.register.clear();
            for (_, agent) in &self.agents {
                self.register.insert(agent.get_hash());
            }
        }
    }

    pub fn contains_score(&self, score: isize) -> bool {
        self.agents.contains_key(&score)
    }

    pub fn will_accept(&self, agent: &Agent<Gene>) -> bool {
        if self.unique_agents {
            return !self.register.contains(&agent.get_hash());
        }
        true
    }

    pub fn get_scores(&self) -> Vec<isize> {
        self.agents.keys().map(|k| *k).collect()
    }

    pub fn get_random_score(&self) -> isize {
        let mut rng = rand::thread_rng();
        self.get_scores()[rng.gen_range(0, self.len())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let population: Population<u8> = Population::new_empty(false);
        assert_eq!(0, population.len());
        assert_eq!(0, population.get_agents().len());
        assert_eq!(0, population.get_scores().len());
    }

    fn get_score_index(agent: &Agent<u8>, _data: &u8) -> isize {
        agent.get_genes()[0] as isize
    }

    #[test]
    fn new_with_false_unique() {
        let mut population = Population::new(5, 6, false, &0, &get_score_index);
        assert_eq!(5, population.len());
        assert_eq!(5, population.get_agents().len());
        assert_eq!(5, population.get_scores().len());
        for (_score, agent) in population.get_agents() {
            assert_eq!(6, agent.get_genes().len());
        }

        let random_score = population.get_random_score();
        let agent = population.get(random_score).unwrap().clone();
        assert!(population.will_accept(&agent));
        let mut new_score = 0;
        while population.contains_score(new_score) {
            new_score += 1;
        }

        population.insert(new_score, agent);
        assert_eq!(6, population.len());
        assert_eq!(6, population.get_agents().len());
        assert_eq!(6, population.get_scores().len());
    }

    #[test]
    fn new_with_true_unique() {
        let mut population = Population::new(5, 6, true, &0, &get_score_index);
        assert_eq!(5, population.len());
        assert_eq!(5, population.get_agents().len());
        assert_eq!(5, population.get_scores().len());
        for (_score, agent) in population.get_agents() {
            assert_eq!(6, agent.get_genes().len());
        }

        let random_score = population.get_random_score();
        let agent = population.get(random_score).unwrap().clone();
        assert!(!population.will_accept(&agent));
        let mut new_score = 0;
        while population.contains_score(new_score) {
            new_score += 1;
        }

        population.insert(new_score, agent.clone());
        assert_eq!(5, population.len());
        assert_eq!(5, population.get_agents().len());
        assert_eq!(5, population.get_scores().len());

        population.remove(random_score);
        assert_eq!(4, population.len());
        assert_eq!(4, population.get_agents().len());
        assert_eq!(4, population.get_scores().len());

        population.insert(new_score, agent);
        assert_eq!(5, population.len());
        assert_eq!(5, population.get_agents().len());
        assert_eq!(5, population.get_scores().len());
    }

    #[test]
    fn cull_all_below() {
        let mut population = Population::new(5, 6, true, &0, &get_score_index);
        assert_eq!(5, population.len());
        assert_eq!(5, population.get_agents().len());
        assert_eq!(5, population.get_scores().len());

        let lowest = population.get_scores()[0];
        let second_lowest = population.get_scores()[1];
        let middle = population.get_scores()[2];
        let second_highest = population.get_scores()[3];
        let highest = population.get_scores()[4];
        
        // Ensure ordering is as expected.
        assert!(highest > lowest);

        // Will be used for checking register of hashes was updated.
        let lowest_clone = population.get(lowest).unwrap().clone();
        let highest_clone = population.get(highest).unwrap().clone();

        population.cull_all_below(middle);
        assert_eq!(3, population.len());
        assert_eq!(3, population.get_agents().len());
        assert_eq!(3, population.get_scores().len());

        assert!(!population.contains_score(lowest));
        assert!(!population.contains_score(second_lowest));
        assert!(population.contains_score(middle));
        assert!(population.contains_score(second_highest));
        assert!(population.contains_score(highest));

        let mut new_score = 0;
        while population.contains_score(new_score) {
            new_score += 1;
        }

        // The highest is still in there and so its clone should not be accepted.
        assert!(!population.will_accept(&highest_clone));
        population.insert(new_score, highest_clone);
        assert_eq!(3, population.len());
        assert_eq!(3, population.get_agents().len());
        assert_eq!(3, population.get_scores().len());

        // The lowest is no longer there and so its clone can be accepted.
        assert!(population.will_accept(&lowest_clone));
        population.insert(new_score, lowest_clone);
        assert_eq!(4, population.len());
        assert_eq!(4, population.get_agents().len());
        assert_eq!(4, population.get_scores().len());
    }
}
