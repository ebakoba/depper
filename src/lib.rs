use std::collections::{HashSet, HashMap};
use petgraph::{graph::{DiGraph, NodeIndex}, algo::is_cyclic_directed, visit::{IntoNodeReferences, NodeIndexable}, Direction};
use anyhow::{Result, Ok};

pub struct DependenciesBuilder {
    all_elements: Vec<&'a str>,
    all_dependencies: Vec<&'a str>,
    dependency_map: HashMap<&'a str, (NodeIndex, &'a [&'a str])>
}

impl DependenciesBuilder {
    pub fn add_element(&mut self, name: &'a str, dependecies: &'a [&str]) -> Self {
        self.all_elements.push(name);
        self.all_dependencies.extend(dependecies);
        self.dependency_map.insert(name, (node, dependecies));
        self
    }

    pub fn build(&self) -> 
}

pub struct Dependencies<'a> {
    all_elements: Vec<&'a str>,
    all_dependencies: Vec<&'a str>,
    graph: DiGraph<&'a str, ()>,
    dependency_map: HashMap<&'a str, (NodeIndex, &'a [&'a str])>
}

impl<'a> Dependencies<'a> {
    pub fn build() -> DependenciesBuilder {
        DependenciesBuilder {
            all_elements: vec![],
            all_dependencies: vec![],
            dependency_map: HashMap::new(),
            graph: DiGraph::new(),
        }
    }
    
    pub fn new() -> Self {
        Self {
            all_elements: vec![],
            all_dependencies: vec![],
            dependency_map: HashMap::new(),
            graph: DiGraph::new(),
        }
    }

    pub fn add_element(&mut self, name: &'a str, dependecies: &'a [&str]) {
        self.all_elements.push(name);
        self.all_dependencies.extend(dependecies);
        let node = self.graph.add_node(name);
        self.dependency_map.insert(name, (node, dependecies));
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
        self.all_dependencies.iter().all(|dependency| elements_set.contains(dependency))
    }

    fn no_cyclic_dependencies(&self) -> bool {
        !is_cyclic_directed(&self.graph)
    }

    pub fn generate_tranches(&mut self) -> Result<Vec<Vec<String>>> {
        self.add_edges();
        let mut tranches: Vec<Vec<String>> = vec![];
        let mut traverse_graph = self.graph.clone();
        while traverse_graph.node_count() > 0 {
            let mut node_to_remove: Vec<(NodeIndex, String)> = vec![];
            let mut new_layer: Vec<String> = Vec::new();
            for (node_index, node_name) in traverse_graph.node_references() {
                if traverse_graph.neighbors_directed(node_index, Direction::Outgoing).count() == 0 {
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
        self.graph.clear_edges();
        Ok(tranches)
    }

    pub fn validate(&mut self) -> Result<()> {
        if !self.dependencies_are_met() {
            return Err(anyhow::anyhow!("Some dependencies do not exist as elements"))
        }
        self.add_edges();
        if !self.no_cyclic_dependencies() {
            return Err(anyhow::anyhow!("Cyclic dependency detected"))
        }
        self.graph.clear_edges();

        Ok(())
    }
}

impl<'a> Default for Dependencies<'a> {
    fn default() -> Self {
        Self::new()
    }
}



#[cfg(test)]
mod tests {
    use super::Dependencies;
    
    #[test]
    fn can_validate_simple_tree() {
        let mut dependencies = Dependencies::new();

        dependencies.add_element("a", &["b", "c"]);
        dependencies.add_element("b", &["c"]);
        dependencies.add_element("c", &[]);
        assert!(dependencies.validate().is_ok());
    }

    #[test]
    fn can_validate_complex_tree() {
        let mut dependencies = Dependencies::new();

        dependencies.add_element("a", &["b", "c"]);
        dependencies.add_element("b", &["c"]);
        dependencies.add_element("c", &["d", "e"]);
        dependencies.add_element("d", &["e"]);
        dependencies.add_element("e", &[]);
        assert!(dependencies.validate().is_ok());
    }

    #[test]
    fn detects_missing_dependencies() {
        let mut dependencies = Dependencies::new();

        dependencies.add_element("a", &["b", "c"]);
        dependencies.add_element("b", &["c"]);
        assert_eq!(dependencies.validate().unwrap_err().to_string(), "Some dependencies do not exist as elements");
    }

    #[test]
    fn detects_cyclic_dependencies() {
        let mut dependencies = Dependencies::new();

        dependencies.add_element("a", &["b", "c"]);
        dependencies.add_element("b", &["c"]);
        dependencies.add_element("c", &["a", "b"]);
        assert_eq!(dependencies.validate().unwrap_err().to_string(), "Cyclic dependency detected");
    }

    #[test]
    fn can_divide_into_tranches() {
        let mut dependencies = Dependencies::new();

        dependencies.add_element("b", &["d"]);
        dependencies.add_element("c", &["d"]);
        dependencies.add_element("a", &["d", "e", "y"]);
        dependencies.add_element("d", &["e"]);
        dependencies.add_element("e", &[]);
        dependencies.add_element("y", &[]);

        println!("{:?}", dependencies.generate_tranches().unwrap());
        assert!(dependencies.generate_tranches().is_ok());
    }
}
