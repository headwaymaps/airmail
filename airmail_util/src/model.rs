use std::{
    collections::HashMap,
    convert::TryInto,
    f64::consts::PI,
    fmt,
    io::{self, Write},
    mem,
};

use crate::feature::{Feature, FeatureRefs};
use airmail_lib::model::{Header, PackedModel};
use bstr::ByteSlice;
use cqdb::CQDB;
use fst::SetBuilder;

const CHUNK_SIZE: usize = 12;
const FEATURE_SIZE: usize = 20;

#[inline]
pub(crate) fn unpack_u32(buf: &[u8]) -> io::Result<u32> {
    if buf.len() < 4 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "not enough data for unpacking u32",
        ));
    }
    Ok(u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]))
}

#[inline]
fn unpack_f64(buf: &[u8]) -> io::Result<f64> {
    if buf.len() < 8 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "not enough data for unpacking f64",
        ));
    }
    Ok(f64::from_le_bytes([
        buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
    ]))
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
pub struct Model<'a> {
    buffer: &'a [u8],
    size: u32,
    header: Header,
    labels: CQDB<'a>,
    attrs: CQDB<'a>,
}

impl<'a> fmt::Debug for Model<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Model")
            .field("size", &self.size)
            .field("header", &self.header)
            .field("labels", &self.labels)
            .field("attrs", &self.attrs)
            .finish()
    }
}

impl<'a> Model<'a> {
    /// Create an instance of a model object from a model in memory
    pub fn new(buf: &'a [u8]) -> io::Result<Self> {
        let size = buf.len();
        if size <= mem::size_of::<Header>() {
            return Err(io::Error::new(io::ErrorKind::Other, "invalid model format"));
        }
        let magic = &buf[0..4];
        if magic != b"lCRF" {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "invalid file format, magic mismatch",
            ));
        }
        let mut index = 4;
        let header_size = unpack_u32(&buf[index..])?;
        index += 4;
        let header_type = &buf[index..index + 4];
        index += 4;
        let version = unpack_u32(&buf[index..])?;
        index += 4;
        let num_features = unpack_u32(&buf[index..])?;
        index += 4;
        let num_labels = unpack_u32(&buf[index..])?;
        index += 4;
        let num_attrs = unpack_u32(&buf[index..])?;
        index += 4;
        let off_features = unpack_u32(&buf[index..])?;
        index += 4;
        let off_labels = unpack_u32(&buf[index..])?;
        index += 4;
        let off_attrs = unpack_u32(&buf[index..])?;
        index += 4;
        let off_label_refs = unpack_u32(&buf[index..])?;
        index += 4;
        let off_attr_refs = unpack_u32(&buf[index..])?;
        let header = Header {
            magic: magic.try_into().unwrap(),
            size: header_size,
            r#type: header_type.try_into().unwrap(),
            version,
            num_features,
            num_labels,
            num_attrs,
            off_features,
            off_labels,
            off_attrs,
            off_label_refs,
            off_attr_refs,
        };
        let labels_start = off_labels as usize;
        let labels = CQDB::new(&buf[labels_start..size])?;
        let attrs_start = off_attrs as usize;
        let attrs = CQDB::new(&buf[attrs_start..size])?;
        Ok(Self {
            buffer: buf,
            size: size as u32,
            header,
            labels,
            attrs,
        })
    }

    /// Number of attributes
    pub fn num_attrs(&self) -> u32 {
        self.header.num_attrs
    }

    /// Number of labels
    pub fn num_labels(&self) -> u32 {
        self.header.num_labels
    }

    /// Convert a label ID to label string
    pub fn to_label(&self, lid: u32) -> Option<&str> {
        self.labels.to_str(lid).and_then(|s| s.to_str().ok())
    }

    /// Convert a label string to label ID
    pub fn to_label_id(&self, value: &str) -> Option<u32> {
        self.labels.to_id(value)
    }

    /// Convert a attribute ID to attribute string
    pub fn to_attr(&self, aid: u32) -> Option<&str> {
        self.attrs.to_str(aid).and_then(|s| s.to_str().ok())
    }

    /// Convert a attribute string to attribute ID
    pub fn to_attr_id(&self, value: &str) -> Option<u32> {
        self.attrs.to_id(value)
    }

    pub(crate) fn label_ref(&self, lid: u32) -> io::Result<FeatureRefs> {
        let mut index = self.header.off_label_refs as usize + CHUNK_SIZE;
        index += 4 * lid as usize;
        let offset = unpack_u32(&self.buffer[index..])? as usize;
        let num_features = unpack_u32(&self.buffer[offset..])?;
        let feature_ids = &self.buffer[offset + 4..];
        Ok(FeatureRefs {
            num_features,
            feature_ids,
        })
    }

    pub(crate) fn attr_ref(&self, lid: u32) -> io::Result<FeatureRefs> {
        let mut index = self.header.off_attr_refs as usize + CHUNK_SIZE;
        index += 4 * lid as usize;
        let offset = unpack_u32(&self.buffer[index..])? as usize;
        let num_features = unpack_u32(&self.buffer[offset..])?;
        let feature_ids = &self.buffer[offset + 4..];
        Ok(FeatureRefs {
            num_features,
            feature_ids,
        })
    }

    pub(crate) fn feature(&self, fid: u32) -> io::Result<Feature> {
        let mut index = self.header.off_features as usize + CHUNK_SIZE;
        index += FEATURE_SIZE * fid as usize;
        let r#type = unpack_u32(&self.buffer[index..])?;
        index += 4;
        let source = unpack_u32(&self.buffer[index..])?;
        index += 4;
        let target = unpack_u32(&self.buffer[index..])?;
        index += 4;
        let weight = unpack_f64(&self.buffer[index..])?;
        Ok(Feature {
            r#type,
            source,
            target,
            weight,
        })
    }

    /// Print the model in human-readable format
    pub fn dump<W: Write>(&self, w: &mut W) -> io::Result<()> {
        // Dump the file header
        let header = &self.header;
        // Dump the transition features
        for i in 0..header.num_labels {
            let label_refs = self.label_ref(i)?;
            for j in 0..label_refs.num_features {
                let fid = label_refs.get(j as usize)?;
                let feature = self.feature(fid)?;
                let _source = self.to_label(feature.source).unwrap();
                let _target = self.to_label(feature.target).unwrap();
            }
        }

        // Dump the state transition features
        let mut vocab_predictivity = HashMap::new();
        for i in 0..header.num_attrs {
            let attr_refs = self.attr_ref(i)?;
            for j in 0..attr_refs.num_features {
                let fid = attr_refs.get(j as usize)?;
                let feature = self.feature(fid)?;
                let attr = self.to_attr(feature.source).unwrap();
                if let Some(&mut predictivity) = vocab_predictivity.get_mut(attr) {
                    vocab_predictivity.insert(attr, f64::abs(feature.weight) + predictivity);
                } else {
                    vocab_predictivity.insert(attr, f64::abs(feature.weight));
                }
            }
        }

        let important_attrs = {
            let mut tmp_attrs: Vec<String> = vocab_predictivity
                .iter()
                .map(|(attr, _predictivity)| attr.to_string())
                .collect();
            tmp_attrs.sort();

            tmp_attrs.clone()
        };

        let label_current_order: Vec<String> = self
            .labels
            .iter()
            .map(|label| label.unwrap().1.to_str().unwrap().to_string())
            .collect();

        let vocab_fst_data = {
            let mut fst_builder = SetBuilder::memory();
            important_attrs
                .iter()
                .for_each(|attr| fst_builder.insert(attr.to_string()).unwrap());
            fst_builder.into_inner().unwrap()
        };

        let lex_sorted_weights = {
            let mut all_weights = vec![];

            for i in 0..header.num_attrs {
                let attr_refs = self.attr_ref(i)?;
                for j in 0..attr_refs.num_features {
                    let fid = attr_refs.get(j as usize)?;
                    let feature = self.feature(fid)?;
                    let attr = self.to_attr(feature.source).unwrap();
                    all_weights.push((attr.to_string(), feature.target, feature.weight));
                }
            }
            all_weights.sort_by(|a, b| {
                a.0.partial_cmp(&b.0)
                    .unwrap()
                    .then(a.1.partial_cmp(&b.1).unwrap())
            });
            all_weights
        };

        let mut packed_weights = vec![];
        for (index, (attr, target, weight)) in lex_sorted_weights.iter().enumerate() {
            let attr_has_more = if index + 1 < lex_sorted_weights.len()
                && &lex_sorted_weights[index + 1].0 == attr
            {
                0x8000u16
            } else {
                0
            };
            let curved_weight = f64::powf(f64::abs(*weight), 1.0 / 7.0);
            if *target > 15 {
                println!("Can't pack target into 4 bits");
                panic!();
            }
            let sign_bit = if *weight < 0.0 { 0x400u16 } else { 0u16 };
            let packed = attr_has_more
                | ((*target as u16) << 11)
                | (f64::round(curved_weight * 1023.0) as u16) & 0x3FF
                | sign_bit;
            println!("{}", weight);
            packed_weights.push(packed);
        }

        let unquantized_label_weights = {
            let mut unquantized_label_weights = vec![];
            self.labels.iter().for_each(|label| {
                let feature_refs = self.label_ref(label.unwrap().0).unwrap();
                for i in 0..feature_refs.num_features {
                    let feature_id = feature_refs.get(i as usize).unwrap();
                    let feature = self.feature(feature_id).unwrap();
                    unquantized_label_weights.push((
                        feature.source as u8,
                        feature.target as u8,
                        feature.weight as f32,
                    ));
                }
            });
            unquantized_label_weights
        };

        w.write_all(
            &bincode2::serialize(&PackedModel {
                header: header.clone(),
                packed_attr_weights: packed_weights,
                attr_vocab_fst: vocab_fst_data,
                labels: label_current_order,
                unquantized_label_weights,
            })
            .unwrap(),
        )
        .unwrap();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Model;
    use std::fs;

    #[test]
    fn test_model_new() {
        let buf = fs::read("tests/model.crfsuite").unwrap();
        let model = Model::new(&buf).unwrap();
        assert_eq!(100, model.header.version);
        assert_eq!(0, model.header.num_features);
        assert_eq!(2, model.header.num_labels);
        assert_eq!(3, model.header.num_attrs);

        let _debug = format!("{:?}", model);
    }

    #[test]
    fn test_invalid_model() {
        let buf = b"";
        let model = Model::new(buf);
        assert!(model.is_err());

        let mut buf = fs::read("tests/model.crfsuite").unwrap();
        let offset = std::mem::size_of::<super::Header>();
        let buf = &mut buf[..offset + 10];
        buf[0] = b'L'; // change magic from lCRF to LCRF
        let model = Model::new(buf);
        assert!(model.is_err());
    }
}
