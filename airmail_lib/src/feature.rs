#[derive(Debug, Clone)]
pub struct Feature {
    pub r#type: u32,
    pub source: u32,
    pub target: u32,
    pub weight: f64,
}

/// Feature references
///
/// This is a collection of feature ids used for faster accesses.
#[derive(Debug, Clone)]
pub struct FeatureRefs {
    pub num_features: u32,
    pub feature_ids: Vec<u32>,
}
