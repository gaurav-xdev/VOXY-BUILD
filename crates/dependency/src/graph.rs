use std::collections::{HashMap, VecDeque};

use crate::error::{DependencyError, Result};
use crate::traits::{DependencyNode, DependencySpec};

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    nodes: HashMap<String, DependencyNode>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, spec: &DependencySpec) {
        let node = DependencyNode {
            name: spec.name.clone(),
            depth: 0,
            dependencies: spec.depends_on.clone(),
        };
        self.nodes.insert(spec.name.clone(), node);
        self.recalculate_depths();
    }

    pub fn remove_node(&mut self, name: &str) {
        self.nodes.remove(name);
        for node in self.nodes.values_mut() {
            node.dependencies.retain(|dep| dep != name);
        }
        self.recalculate_depths();
    }

    pub fn has_cycle(&self) -> bool {
        let mut visited = HashMap::new();
        for name in self.nodes.keys() {
            if self.dfs_has_cycle(name, &mut visited) {
                return true;
            }
        }
        false
    }

    fn dfs_has_cycle(&self, name: &str, visited: &mut HashMap<String, bool>) -> bool {
        match visited.get(name) {
            Some(&true) => return true,
            Some(&false) => return false,
            None => {}
        }
        visited.insert(name.to_string(), true);
        if let Some(node) = self.nodes.get(name) {
            for dep in &node.dependencies {
                if self.dfs_has_cycle(dep, visited) {
                    return true;
                }
            }
        }
        visited.insert(name.to_string(), false);
        false
    }

    pub fn topological_sort(&self) -> Result<Vec<String>> {
        if self.has_cycle() {
            let cycle_nodes: Vec<String> = self.nodes.keys().cloned().collect();
            return Err(DependencyError::CycleDetected(format!(
                "Cycle detected among nodes: {:?}",
                cycle_nodes
            )));
        }

        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

        for (name, node) in &self.nodes {
            in_degree.entry(name.clone()).or_insert(0);
            adjacency.entry(name.clone()).or_default();
            for dep in &node.dependencies {
                if self.nodes.contains_key(dep) {
                    adjacency.entry(dep.clone()).or_default().push(name.clone());
                    *in_degree.entry(name.clone()).or_insert(0) += 1;
                }
            }
        }

        let mut queue = VecDeque::new();
        for (name, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(name.clone());
            }
        }

        let mut sorted = Vec::new();
        while let Some(name) = queue.pop_front() {
            sorted.push(name.clone());
            if let Some(neighbors) = adjacency.get(&name) {
                for neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        Ok(sorted)
    }

    fn recalculate_depths(&mut self) {
        let sorted = self.topological_sort().unwrap_or_default();
        for name in &sorted {
            let depth = self
                .nodes
                .get(name)
                .map(|n| {
                    n.dependencies
                        .iter()
                        .filter_map(|dep| self.nodes.get(dep))
                        .map(|dep| dep.depth + 1)
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            if let Some(node) = self.nodes.get_mut(name) {
                node.depth = depth;
            }
        }
    }

    pub fn dependencies_of(&self, name: &str) -> Vec<String> {
        self.nodes
            .get(name)
            .map(|n| n.dependencies.clone())
            .unwrap_or_default()
    }

    pub fn dependents_of(&self, name: &str) -> Vec<String> {
        self.nodes
            .iter()
            .filter(|(_, node)| node.dependencies.contains(&name.to_string()))
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub fn all_nodes(&self) -> Vec<&DependencyNode> {
        self.nodes.values().collect()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::SchedulingOrder;

    fn spec(name: &str, deps: Vec<&str>) -> DependencySpec {
        DependencySpec {
            name: name.to_string(),
            depends_on: deps.iter().map(|s| s.to_string()).collect(),
            provides: vec![],
            required: true,
            order: SchedulingOrder::Topological,
            timeout_seconds: 30,
            auto_restart: false,
        }
    }

    #[test]
    fn test_graph_add_and_remove_nodes() {
        let mut graph = DependencyGraph::new();
        assert!(graph.is_empty());

        graph.add_node(&spec("a", vec![]));
        graph.add_node(&spec("b", vec!["a"]));
        assert_eq!(graph.len(), 2);

        graph.remove_node("a");
        assert_eq!(graph.len(), 1);
        let deps = graph.dependencies_of("b");
        assert!(deps.is_empty());
    }

    #[test]
    fn test_topological_sort_simple_dag() {
        let mut graph = DependencyGraph::new();
        graph.add_node(&spec("a", vec![]));
        graph.add_node(&spec("b", vec!["a"]));
        graph.add_node(&spec("c", vec!["b"]));

        let sorted = graph.topological_sort().unwrap();
        let pos = |name: &str| sorted.iter().position(|s| s == name).unwrap();
        assert!(pos("a") < pos("b"));
        assert!(pos("b") < pos("c"));
    }

    #[test]
    fn test_topological_sort_independent_nodes() {
        let mut graph = DependencyGraph::new();
        graph.add_node(&spec("a", vec![]));
        graph.add_node(&spec("b", vec![]));
        graph.add_node(&spec("c", vec![]));

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);
        for name in &["a", "b", "c"] {
            assert!(sorted.contains(&name.to_string()));
        }
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();
        graph.add_node(&spec("a", vec!["b"]));
        graph.add_node(&spec("b", vec!["c"]));
        graph.add_node(&spec("c", vec!["a"]));

        assert!(graph.has_cycle());
        assert!(graph.topological_sort().is_err());
    }

    #[test]
    fn test_cycle_detection_self_loop() {
        let mut graph = DependencyGraph::new();
        graph.add_node(&spec("a", vec!["a"]));
        assert!(graph.has_cycle());
    }

    #[test]
    fn test_dependencies_of() {
        let mut graph = DependencyGraph::new();
        graph.add_node(&spec("a", vec![]));
        graph.add_node(&spec("b", vec!["a"]));

        assert!(graph.dependencies_of("a").is_empty());
        assert_eq!(graph.dependencies_of("b"), vec!["a"]);
    }

    #[test]
    fn test_dependents_of() {
        let mut graph = DependencyGraph::new();
        graph.add_node(&spec("a", vec![]));
        graph.add_node(&spec("b", vec!["a"]));
        graph.add_node(&spec("c", vec!["a"]));

        let mut deps = graph.dependents_of("a");
        deps.sort();
        assert_eq!(deps, vec!["b", "c"]);
    }

    #[test]
    fn test_all_nodes() {
        let mut graph = DependencyGraph::new();
        graph.add_node(&spec("a", vec![]));
        graph.add_node(&spec("b", vec!["a"]));

        let nodes = graph.all_nodes();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn test_graph_default() {
        let graph = DependencyGraph::default();
        assert!(graph.is_empty());
    }
}
