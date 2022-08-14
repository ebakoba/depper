use anyhow::{Ok, Result};
use petgraph::{
    algo::is_cyclic_directed,
    graph::{DiGraph, NodeIndex},
    visit::{IntoNodeReferences, NodeIndexable},
    Direction,
};
use std::collections::{HashMap, HashSet};

pub struct DependenciesBuilder<'a> {
    all_elements: Vec<&'a str>,
    all_dependencies: Vec<&'a str>,
    graph: DiGraph<&'a str, ()>,
    dependency_map: HashMap<&'a str, (NodeIndex, &'a [&'a str])>,
}

impl<'a> DependenciesBuilder<'a> {
    pub fn add_element(mut self, name: &'a str, dependecies: &'a [&str]) -> Self {
        self.all_elements.push(name);
        self.all_dependencies.extend(dependecies);
        let node = self.graph.add_node(name);
        self.dependency_map.insert(name, (node, dependecies));
        self
    }

    fn add_edges(&mut self) {
        for (node, dependencies) in self.dependency_map.values() {
            for dependency in *dependencies {
                let dependency_node = self.dependency_map[dependency].0;
                self.graph.add_edge(*node, dependency_node, ());
            }
        }
    }

    fn dependencies_are_met(&self) -> bool {
        let elements_set: HashSet<_> = self.all_elements.iter().collect();
        self.all_dependencies
            .iter()
            .all(|dependency| elements_set.contains(dependency))
    }

    fn no_cyclic_dependencies(&self) -> bool {
        !is_cyclic_directed(&self.graph)
    }

    fn validate(&mut self) -> Result<()> {
        if !self.dependencies_are_met() {
            return Err(anyhow::anyhow!(
                "Some dependencies do not exist as elements"
            ));
        }
        self.add_edges();
        if !self.no_cyclic_dependencies() {
            return Err(anyhow::anyhow!("Cyclic dependency detected"));
        }
        self.graph.clear_edges();

        Ok(())
    }

    pub fn build(&mut self) -> Result<Dependencies> {
        self.validate()?;
        self.add_edges();
        Ok(Dependencies {
            graph: self.graph.clone(),
        })
    }
}

#[derive(Debug)]
pub struct Dependencies<'a> {
    graph: DiGraph<&'a str, ()>,
}

impl<'a> Dependencies<'a> {
    pub fn generate_tranches(&self) -> Result<Vec<Vec<String>>> {
        let mut tranches: Vec<Vec<String>> = vec![];
        let mut traverse_graph = self.graph.clone();
        while traverse_graph.node_count() > 0 {
            let mut node_to_remove: Vec<(NodeIndex, String)> = vec![];
            let mut new_layer: Vec<String> = Vec::new();
            for (node_index, node_name) in traverse_graph.node_references() {
                if traverse_graph
                    .neighbors_directed(node_index, Direction::Outgoing)
                    .count()
                    == 0
                {
                    node_to_remove.push((node_index, node_name.to_string()));
                }
            }
            for (node_index, node_name) in node_to_remove {
                let adjusted_index = traverse_graph.to_index(node_index) - new_layer.len();
                traverse_graph.remove_node(traverse_graph.from_index(adjusted_index));

                new_layer.push(node_name.to_string())
            }
            tranches.push(new_layer);
        }
        Ok(tranches)
    }

    pub fn builder() -> DependenciesBuilder<'a> {
        DependenciesBuilder {
            all_elements: Vec::new(),
            all_dependencies: Vec::new(),
            graph: DiGraph::new(),
            dependency_map: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Dependencies;

    #[test]
    fn can_validate_simple_tree() {
        let mut dependencies_builder = Dependencies::builder()
            .add_element("a", &["b", "c"])
            .add_element("b", &["c"])
            .add_element("c", &[]);

        assert!(dependencies_builder.build().is_ok());
    }

    #[test]
    fn can_validate_complex_tree() {
        let mut dependencies_builder = Dependencies::builder()
            .add_element("a", &["b", "c"])
            .add_element("b", &["c"])
            .add_element("c", &["d", "e"])
            .add_element("d", &["e"])
            .add_element("e", &[]);

        assert!(dependencies_builder.build().is_ok());
    }

    #[test]
    fn detects_missing_dependencies() {
        let mut dependencies_builder = Dependencies::builder()
            .add_element("a", &["b", "c"])
            .add_element("b", &["c"]);

        assert_eq!(
            dependencies_builder.build().unwrap_err().to_string(),
            "Some dependencies do not exist as elements"
        );
    }

    #[test]
    fn detects_cyclic_dependencies() {
        let mut dependencies_builder = Dependencies::builder()
            .add_element("a", &["b", "c"])
            .add_element("b", &["c"])
            .add_element("c", &["a", "b"]);

        assert_eq!(
            dependencies_builder.build().unwrap_err().to_string(),
            "Cyclic dependency detected"
        );
    }

    #[test]
    fn can_divide_into_tranches() {
        let mut dependencies_builder = Dependencies::builder()
            .add_element("b", &["d"])
            .add_element("c", &["d"])
            .add_element("a", &["d", "e", "y"])
            .add_element("d", &["e"])
            .add_element("e", &[])
            .add_element("y", &[]);

        let dependencies = dependencies_builder.build().unwrap();

        assert!(dependencies.generate_tranches().is_ok());
    }
}
