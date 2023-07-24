use std::cmp::Ordering;

use super::Factory;
use petgraph::{
    graphmap::DiGraphMap,
    visit::{Topo, Walker},
};

#[derive(Debug, Clone, PartialEq)]
pub struct CalcStep {
    pub factory: Factory<'static>,
    pub amount: f64,
}

impl CalcStep {
    pub fn produced_per_sec(&self) -> Vec<(String, f64)> {
        self.factory
            .produced_per_sec()
            .into_iter()
            .map(|(name, amount)| (name, amount * self.amount))
            .collect()
    }

    pub fn consumed_per_sec(&self) -> Vec<(String, f64)> {
        self.factory
            .consumed_per_sec()
            .into_iter()
            .map(|(name, amount)| (name, amount * self.amount))
            .collect()
    }

    pub fn machine_name(&self) -> String {
        self.factory.name()
    }
}

pub fn graph_sort(input: Vec<CalcStep>) -> Vec<CalcStep> {
    let mut dep_graph = DiGraphMap::<usize, ()>::with_capacity(input.len(), 0);
    for i in 0..input.len() {
        dep_graph.add_node(i);
    }
    for (i, step) in input.iter().enumerate() {
        for (j, step2) in input.iter().enumerate().filter(|(_, st)| *st != step) {
            match step.factory.sort_by(&step2.factory) {
                Ordering::Less => {
                    dep_graph.add_edge(i, j, ());
                }
                Ordering::Greater => {
                    dep_graph.add_edge(j, i, ());
                }
                Ordering::Equal => {}
            };
        }
    }
    let indices: Vec<usize> = Walker::iter(Topo::new(&dep_graph), &dep_graph).collect();
    indices.into_iter().map(|i| input[i].clone()).collect()
}
