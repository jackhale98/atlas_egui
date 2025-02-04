// src/utils.rs
use crate::config::{Component, Feature};

pub fn find_feature<'a>(components: &'a [Component], component_name: &str, feature_name: &str) -> Option<&'a Feature> {
    components.iter()
        .find(|c| c.name == component_name)?
        .features.iter()
        .find(|f| f.name == feature_name)
}
