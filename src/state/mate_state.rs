// src/state/mate_state.rs
use petgraph::graph::{Graph, NodeIndex};
use std::collections::HashMap;
use crate::config::mate::Mate;
use crate::config::Component;

#[derive(Debug)]
pub struct MateState {
    pub mates: Vec<Mate>,
    pub dependency_graph: Graph<String, String>,
    pub feature_nodes: HashMap<(String, String), NodeIndex>, // (component_id, feature_id) -> node_index
    pub filter: Option<MateFilter>,
}

#[derive(Debug, Clone)]
pub enum MateFilter {
    Component(String),
    Feature(String, String), // (component_name, feature_name)
}

impl Default for MateState {
    fn default() -> Self {
        Self {
            mates: Vec::new(),
            dependency_graph: Graph::new(),
            feature_nodes: HashMap::new(),
            filter: None,
        }
    }
}


pub fn get_component_by_name<'a>(components: &'a [Component], name: &str) -> Option<&'a Component> {
    components
        .iter()
        .find(|c| c.name == name)
}

impl MateState {
    pub fn update_dependency_graph(&mut self, components: &[Component]) {
        // Clear existing graph
        self.dependency_graph = Graph::new();
        self.feature_nodes.clear();

        // Add nodes for all features
        for component in components {
            for feature in &component.features {
                let node_id = self.dependency_graph.add_node(feature.name.clone());
                self.feature_nodes.insert(
                    (component.name.clone(), feature.name.clone()),
                    node_id
                );
            }
        }

        // Add edges for all mates
        for mate in &self.mates {
            if let (Some(&node_a), Some(&node_b)) = (
                self.feature_nodes.get(&(mate.component_a.clone(), mate.feature_a.clone())),
                self.feature_nodes.get(&(mate.component_b.clone(), mate.feature_b.clone()))
            ) {
                // Add bidirectional edges
                self.dependency_graph.add_edge(
                    node_a,
                    node_b,
                    format!("{:?}", mate.fit_type)
                );
            }
        }
    }
    pub fn filtered_mates(&self) -> Vec<&Mate> {
        match &self.filter {
            Some(MateFilter::Component(comp_name)) => {
                self.mates.iter()
                    .filter(|mate| {
                        mate.component_a == *comp_name || mate.component_b == *comp_name
                    })
                    .collect()
            },
            Some(MateFilter::Feature(comp_name, feat_name)) => {
                self.mates.iter()
                    .filter(|mate| {
                        (mate.component_a == *comp_name && mate.feature_a == *feat_name) ||
                        (mate.component_b == *comp_name && mate.feature_b == *feat_name)
                    })
                    .collect()
            },
            None => self.mates.iter().collect()
        }
    }

    pub fn get_related_mates(&self, component: &str, feature: &str) -> Vec<&Mate> {
        self.mates.iter()
            .filter(|mate| {
                (mate.component_a == component && mate.feature_a == feature) ||
                (mate.component_b == component && mate.feature_b == feature)
            })
            .collect()
    }

    pub fn get_feature_dependencies(&self, component: &str, feature: &str) -> Vec<(String, String)> {
        if let Some(&node_idx) = self.feature_nodes.get(&(component.to_string(), feature.to_string())) {
            let mut deps = Vec::new();

            // Get all neighbors (both incoming and outgoing edges)
            for neighbor in self.dependency_graph.neighbors_undirected(node_idx) {
                // Find the component and feature name for this node
                if let Some((key, _)) = self.feature_nodes.iter()
                    .find(|(_, &idx)| idx == neighbor) {
                    deps.push((key.0.clone(), key.1.clone()));
                }
            }

            deps
        } else {
            Vec::new()
        }
    }
}

