//! [![github]](https://github.com/ebakoba/depper)&ensp;[![crates-io]](https://crates.io/crates/depper)&ensp;[![docs-rs]](https://docs.rs/depper)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K
//!
//! <br>
//!
//! Library for detecting dependency cycles and finding missing dependencies. It also allows to sort
//! dependencies into tranches, which can be used as a hierarchy dependency resolution.
//!
//!
//! <br>
//!
//! # Details
//!
//! - It exposes two structs `DependencyBuilder` and `Dependencies`. First is for building up the list of dependencies
//! and building (calling the `.build()` function also validates the entire list) the second struct. Second is for
//! generating tranches of dependencies for deployment hierarchies.
//!
//!
//!   ```
//!   use depper::Dependencies;
//!
//!   let mut dependencies_builder = Dependencies::builder()
//!     .add_element("b".to_string(), vec!["d".to_string()])
//!     .add_element("c".to_string(), vec!["d".to_string()])
//!     .add_element("a".to_string(), vec!["d".to_string(), "e".to_string(), "y".to_string()])
//!     .add_element("d".to_string(), vec!["e".to_string()])
//!     .add_element("e".to_string(), vec![])
//!     .add_element("y".to_string(), vec![]);
//!     
//!   // Calling the `.build()` function validates the list of dependencies.
//!   let dependencies = dependencies_builder.build().unwrap();
//!    
//!   // The `.tranches()` function returns a list of tranches.
//!   println!("{:?}", dependencies.generate_tranches().unwrap());
//!   ```
//!
//!   ```console
//!   [["e", "y"], ["d"], ["b", "c", "a"]]
//!   ```
//!

use anyhow::{Ok, Result};
use petgraph::{
    algo::is_cyclic_directed,
    graph::{DiGraph, NodeIndex},
    visit::{IntoNodeReferences, NodeIndexable},
    Direction, Graph,
};
use std::collections::{HashMap, HashSet};

pub struct DependenciesBuilder {
    all_elements: Vec<String>,
    all_dependencies: Vec<String>,
    graph: DiGraph<String, ()>,
    dependency_map: HashMap<String, (NodeIndex, Vec<String>)>,
}

impl DependenciesBuilder {
    pub fn add_element(mut self, name: String, dependecies: Vec<String>) -> Self {
        self.all_dependencies.extend(dependecies.clone());

        if let Some((graph_node, _)) = self.dependency_map.get(&name) {
            self.dependency_map
                .insert(name, (graph_node.to_owned(), dependecies));
        } else {
            self.all_elements.push(name.clone());
            let node = self.graph.add_node(name.clone());
            self.dependency_map.insert(name, (node, dependecies));
        }
        self
    }

    fn add_edges(&mut self) {
        for (node, dependencies) in self.dependency_map.values() {
            for dependency in dependencies {
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
pub struct Dependencies {
    graph: DiGraph<String, ()>,
}

impl Dependencies {
    fn find_node_by_name(graph: Graph<String, ()>, name: &str) -> Option<NodeIndex> {
        for (node_index, node_name) in graph.node_references() {
            if node_name == name {
                return Some(node_index);
            }
        }
        None
    }

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
            for (_, node_name) in node_to_remove {
                let node_index =
                    Dependencies::find_node_by_name(traverse_graph.clone(), &node_name)
                        .ok_or(anyhow::anyhow!("Node not found"))?;
                traverse_graph
                    .remove_node(traverse_graph.from_index(traverse_graph.to_index(node_index)));

                new_layer.push(node_name.to_string())
            }
            tranches.push(new_layer);
        }
        Ok(tranches)
    }

    pub fn builder() -> DependenciesBuilder {
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
            .add_element("a".to_string(), vec!["b".to_string(), "c".to_string()])
            .add_element("b".to_string(), vec!["c".to_string()])
            .add_element("c".to_string(), vec![]);

        assert!(dependencies_builder.build().is_ok());
    }

    #[test]
    fn can_validate_complex_tree() {
        let mut dependencies_builder = Dependencies::builder()
            .add_element("a".to_string(), vec!["b".to_string(), "c".to_string()])
            .add_element("b".to_string(), vec!["c".to_string()])
            .add_element("c".to_string(), vec!["d".to_string(), "e".to_string()])
            .add_element("d".to_string(), vec!["e".to_string()])
            .add_element("e".to_string(), vec![]);

        assert!(dependencies_builder.build().is_ok());
    }

    #[test]
    fn detects_missing_dependencies() {
        let mut dependencies_builder = Dependencies::builder()
            .add_element("a".to_string(), vec!["b".to_string(), "c".to_string()])
            .add_element("b".to_string(), vec!["c".to_string()]);

        assert_eq!(
            dependencies_builder.build().unwrap_err().to_string(),
            "Some dependencies do not exist as elements"
        );
    }

    #[test]
    fn detects_cyclic_dependencies() {
        let mut dependencies_builder = Dependencies::builder()
            .add_element("a".to_string(), vec!["b".to_string(), "c".to_string()])
            .add_element("b".to_string(), vec!["c".to_string()])
            .add_element("c".to_string(), vec!["a".to_string(), "b".to_string()]);

        assert_eq!(
            dependencies_builder.build().unwrap_err().to_string(),
            "Cyclic dependency detected"
        );
    }

    #[test]
    fn can_divide_into_tranches() {
        let mut dependencies_builder = Dependencies::builder()
            .add_element("b".to_string(), vec!["d".to_string()])
            .add_element("c".to_string(), vec!["d".to_string()])
            .add_element(
                "a".to_string(),
                vec!["d".to_string(), "e".to_string(), "y".to_string()],
            )
            .add_element("d".to_string(), vec!["e".to_string()])
            .add_element("e".to_string(), vec![])
            .add_element("y".to_string(), vec![]);

        let dependencies = dependencies_builder.build().unwrap();

        insta::assert_debug_snapshot!(dependencies.generate_tranches().unwrap());
    }

    #[test]
    fn can_update_dependecies_later() {
        let mut dependencies_builder = Dependencies::builder()
            .add_element("b".to_string(), vec!["d".to_string()])
            .add_element("c".to_string(), vec!["d".to_string()])
            .add_element(
                "a".to_string(),
                vec!["d".to_string(), "e".to_string(), "y".to_string()],
            )
            .add_element("d".to_string(), vec!["e".to_string()])
            .add_element("e".to_string(), vec![])
            .add_element("y".to_string(), vec![])
            .add_element("e".to_string(), vec!["y".to_string()]);

        let dependencies = dependencies_builder.build().unwrap();

        insta::assert_debug_snapshot!(dependencies.generate_tranches().unwrap());
    }

    #[test]
    fn dependecies_are_order_insensitive() {
        let mut ordered_dependencies_builder = Dependencies::builder()
            .add_element("machine1".to_string(), vec!["network1".to_string()])
            .add_element(
                "machine2".to_string(),
                vec!["network1".to_string(), "network2".to_string()],
            )
            .add_element("machine3".to_string(), vec!["network2".to_string()])
            .add_element("network1".to_string(), vec![])
            .add_element("network2".to_string(), vec![]);

        let ordered_dependecies = ordered_dependencies_builder.build().unwrap();

        let mut dependencies_builder = Dependencies::builder()
            .add_element("machine1".to_string(), vec!["network1".to_string()])
            .add_element("network1".to_string(), vec![])
            .add_element("network2".to_string(), vec![])
            .add_element(
                "machine2".to_string(),
                vec!["network1".to_string(), "network2".to_string()],
            )
            .add_element("machine3".to_string(), vec!["network2".to_string()]);

        let dependencies = dependencies_builder.build().unwrap();

        println!("{:?}", ordered_dependecies.generate_tranches());
        println!("{:?}", dependencies.generate_tranches());
        assert_eq!(
            ordered_dependecies.generate_tranches().unwrap().len(),
            dependencies.generate_tranches().unwrap().len()
        );
    }
}
