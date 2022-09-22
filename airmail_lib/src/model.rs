use std::{
    collections::{HashMap, HashSet},
    fmt, io,
};

use fst::raw::Fst;
use serde::{Deserialize, Serialize};

use crate::feature::{Feature, FeatureRefs};
use crate::tagger::Tagger;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct Header {
    pub magic: [u8; 4],
    pub size: u32,
    pub r#type: [u8; 4],
    pub version: u32,
    pub num_features: u32,
    pub num_labels: u32,
    pub num_attrs: u32,
    pub off_features: u32,
    pub off_labels: u32,
    pub off_attrs: u32,
    pub off_label_refs: u32,
    pub off_attr_refs: u32,
}

#[derive(Debug, Clone)]
#[repr(C)]
struct FeatureRefHeader {
    chunk: [u8; 4],
    size: u32,
    num: u32,
    offsets: [u32; 1],
}

#[derive(Debug, Clone)]
#[repr(C)]
struct FeatureHeader {
    chunk: [u8; 4],
    size: u32,
    num: u32,
}

/// The CRF model
#[derive(Clone)]
pub struct Model {
    header: Header,
    attr_vocab_fst: Fst<Vec<u8>>,
    attr_vocab: HashMap<String, u32>,
    label_vocab: HashMap<String, u32>,
    attr_vocab_reverse: HashMap<u32, String>,
    label_vocab_reverse: HashMap<u32, String>,
    feature_weights: HashMap<u32, (u32, u32, f32)>,
    feature_indices_for_source: HashMap<u32, Vec<u32>>,
    label_features: HashSet<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct PackedModel {
    pub header: Header,
    pub attr_vocab_fst: Vec<u8>,
    pub labels: Vec<String>,
    pub unquantized_label_weights: Vec<(u8, u8, f32)>,
    pub packed_attr_weights: Vec<u16>,
}

impl fmt::Debug for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Model")
            .field("header", &self.header)
            .finish()
    }
}

impl From<PackedModel> for Model {
    fn from(packed: PackedModel) -> Self {
        let (label_vocab, attr_vocab, attr_vocab_fst) = {
            let attr_vocab_fst = Fst::new(packed.attr_vocab_fst).unwrap();

            let mut vocab_idx = 0u32;
            let label_vocab: HashMap<String, u32> = packed
                .labels
                .iter()
                .map(|key| {
                    let pair = (key.clone(), vocab_idx);
                    vocab_idx += 1;
                    pair
                })
                .collect();
            let attr_vocab: HashMap<String, u32> = attr_vocab_fst
                .stream()
                .into_str_keys()
                .unwrap()
                .iter()
                .map(|key| {
                    let pair = (key.clone(), vocab_idx);
                    vocab_idx += 1;
                    pair
                })
                .collect();
            (label_vocab, attr_vocab, attr_vocab_fst)
        };
        let label_vocab_reverse: HashMap<u32, String> = label_vocab
            .iter()
            .map(|(key, id)| (*id, key.clone()))
            .collect();
        let attr_vocab_reverse: HashMap<u32, String> = attr_vocab
            .iter()
            .map(|(key, id)| (*id, key.clone()))
            .collect();

        let (feature_weights, feature_indices_for_source, label_features) = {
            let mut feature_weight_id = 0u32;
            let mut feature_weights: HashMap<u32, (u32, u32, f32)> = HashMap::new();
            let mut feature_indices_for_source: HashMap<u32, Vec<u32>> = HashMap::new();
            let mut label_features = HashSet::new();
            packed
                .unquantized_label_weights
                .iter()
                .for_each(|(source, target, weight)| {
                    feature_weights
                        .insert(feature_weight_id, (*source as u32, *target as u32, *weight));
                    if let Some(v) = feature_indices_for_source.get_mut(&(*source as u32)) {
                        v.push(feature_weight_id);
                    } else {
                        feature_indices_for_source.insert(*source as u32, vec![feature_weight_id]);
                    }
                    label_features.insert(feature_weight_id);
                    feature_weight_id += 1;
                });
            // The length of the labels is the index of the first attr.
            let mut source = label_vocab.len() as u32;
            packed
                .packed_attr_weights
                .iter()
                .for_each(|packed_feature| {
                    let (has_more, target, weight) = Model::unpack_feature(*packed_feature);
                    feature_weights.insert(feature_weight_id, (source, target, weight));
                    if let Some(v) = feature_indices_for_source.get_mut(&source) {
                        v.push(feature_weight_id);
                    } else {
                        feature_indices_for_source.insert(source, vec![feature_weight_id]);
                    }
                    if !has_more {
                        source += 1;
                    }
                    feature_weight_id += 1;
                });
            (feature_weights, feature_indices_for_source, label_features)
        };

        Model {
            header: packed.header.clone(),
            attr_vocab_fst,
            attr_vocab,
            label_vocab,
            attr_vocab_reverse,
            label_vocab_reverse,
            feature_weights,
            feature_indices_for_source,
            label_features,
        }
    }
}

impl<'a> Model {
    fn unpack_feature(packed_feature: u16) -> (bool, u32, f32) {
        let has_more = packed_feature & 0x8000 != 0;
        let target = ((packed_feature >> 11) & 0xF) as u32;
        let raw_packed_weight = (packed_feature & 0x3FF) as f64;
        let weight_curved = raw_packed_weight as f64 / 1023.0;
        let uncurved_weight = f64::powf(weight_curved, 7.0);
        if packed_feature & 0x400 == 0 {
            (has_more, target, uncurved_weight as f32)
        } else {
            (has_more, target, -uncurved_weight as f32)
        }
    }

    pub fn get_vocab(&self) -> Fst<Vec<u8>> {
        return self.attr_vocab_fst.clone();
    }

    /// Number of attributes
    pub fn num_attrs(&self) -> u32 {
        self.attr_vocab.len() as u32
    }

    /// Number of labels
    pub fn num_labels(&self) -> u32 {
        self.label_vocab.len() as u32
    }

    /// Convert a label ID to label string
    pub fn to_label(&self, lid: u32) -> Option<&str> {
        match self.label_vocab_reverse.get(&lid) {
            Some(s) => Some(s.as_str()),
            None => None,
        }
    }

    /// Convert a label string to label ID
    pub fn to_label_id(&self, value: &str) -> Option<u32> {
        match self.label_vocab.get(value) {
            Some(id) => Some(*id),
            None => None,
        }
    }

    /// Convert a attribute ID to attribute string
    pub fn to_attr(&self, aid: u32) -> Option<&str> {
        match self.attr_vocab_reverse.get(&aid) {
            Some(s) => Some(s.as_str()),
            None => None,
        }
    }

    /// Convert a attribute string to attribute ID
    pub fn to_attr_id(&self, value: &str) -> Option<u32> {
        match self.attr_vocab.get(value) {
            Some(id) => Some(*id),
            None => None,
        }
    }

    pub(crate) fn label_ref(&self, lid: u32) -> io::Result<FeatureRefs> {
        Ok(FeatureRefs {
            num_features: self.feature_indices_for_source.get(&lid).unwrap().len() as u32,
            feature_ids: self.feature_indices_for_source.get(&lid).unwrap().clone(),
        })
    }

    pub(crate) fn attr_ref(&self, aid: u32) -> io::Result<FeatureRefs> {
        Ok(FeatureRefs {
            num_features: self.feature_indices_for_source.get(&aid).unwrap().len() as u32,
            feature_ids: self.feature_indices_for_source.get(&aid).unwrap().clone(),
        })
    }

    pub(crate) fn feature(&self, fid: u32) -> io::Result<Feature> {
        if let Some((source, target, weight)) = self.feature_weights.get(&fid) {
            Ok(Feature {
                r#type: if self.label_features.contains(&fid) {
                    1
                } else {
                    0
                },
                source: *source,
                target: *target,
                weight: *weight as f64,
            })
        } else {
            panic!();
        }
    }

    /// Get a new tagger
    pub fn tagger(&'a self) -> io::Result<Tagger<'a>> {
        Tagger::new(self)
    }
}
