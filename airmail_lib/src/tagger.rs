use std::io;

use rand::{thread_rng, Rng};

use crate::context::{Context, Flag, Reset};
use crate::dataset::{self, Instance, Item};
use crate::model::Model;

#[derive(Debug, Clone, Copy)]
enum Level {
    None,
    Set,
    #[allow(dead_code)]
    AlphaBeta,
}

/// Tuple of attribute and its value
#[derive(Debug, Clone)]
pub struct Attribute {
    /// Attribute name
    pub name: String,
    /// Value of the attribute
    pub value: f64,
}

/// The tagger provides the functionality for predicting label sequences for input sequences using a model
#[derive(Debug, Clone)]
pub struct Tagger<'a> {
    /// CRF model
    model: &'a Model,
    /// CRF context
    context: Context,
    /// Number of distinct output labels
    num_labels: u32,
    /// Number of distinct attributes
    _num_attrs: u32,
    level: Level,
    randomness: f64,
}

impl Attribute {
    pub fn new<T: Into<String>>(name: T, value: f64) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}

impl<'a> Tagger<'a> {
    pub(crate) fn new(model: &'a Model) -> io::Result<Self> {
        let num_labels = model.num_labels();
        let num_attrs = model.num_attrs();
        let mut context = Context::new(Flag::VITERBI | Flag::MARGINALS, num_labels, 0);
        context.reset(Reset::TRANS);
        let mut tagger = Self {
            model,
            context,
            num_labels,
            _num_attrs: num_attrs,
            level: Level::None,
            randomness: 0f64,
        };
        tagger.transition_score()?;
        tagger.context.exp_transition();
        Ok(tagger)
    }

    pub fn len(&self) -> usize {
        self.context.num_items as usize
    }

    pub fn is_empty(&self) -> bool {
        self.context.num_items == 0
    }

    /// Predict the label sequence for the item sequence.
    pub fn tag<T: AsRef<[Attribute]>>(
        &mut self,
        xseq: &[T],
        randomness: f64,
    ) -> io::Result<(Vec<&str>, f64)> {
        if xseq.is_empty() {
            return Ok((Vec::new(), 1.0));
        }
        self.randomness = randomness;
        self.set(xseq)?;
        let (label_ids, score) = self.viterbi();
        let mut labels = Vec::with_capacity(label_ids.len());
        for id in label_ids {
            let label = self.model.to_label(id).unwrap();
            labels.push(label);
        }
        Ok((labels, score))
    }

    /// Set an instance (item sequence) for future calls of `tag`, `probability` and `marginal` methods
    pub fn set<T: AsRef<[Attribute]>>(&mut self, xseq: &[T]) -> io::Result<()> {
        let mut instance = Instance::with_capacity(xseq.len());
        for item in xseq {
            let item: Item = item
                .as_ref()
                .iter()
                .filter_map(|x| {
                    self.model
                        .to_attr_id(&x.name)
                        .map(|id| dataset::Attribute::new(id, x.value))
                })
                .collect();
            instance.push(item, 0);
        }
        self.context.set_num_items(instance.num_items);
        self.context.reset(Reset::STATE);
        self.state_score(&instance)?;
        self.level = Level::Set;
        Ok(())
    }

    fn transition_score(&mut self) -> io::Result<()> {
        // Compute transition scores between two labels
        let l = self.num_labels as usize;
        for i in 0..l {
            let trans = &mut self.context.trans[l * i..];
            let edge = self.model.label_ref(i as u32)?;
            for fid in edge.feature_ids {
                // Transition feature from #i to #(feature.target)
                let feature = self.model.feature(fid)?;
                if self.randomness == 0.0 {
                    trans[feature.target as usize] = feature.weight;
                } else {
                    trans[feature.target as usize] = feature.weight
                        * thread_rng().gen_range((1.0 - self.randomness)..(1.0 + self.randomness));
                }
            }
        }
        Ok(())
    }

    fn state_score(&mut self, instance: &Instance) -> io::Result<()> {
        // Loop over the items in the sequence
        for t in 0..instance.num_items as usize {
            let item = &instance.items[t];
            let state = &mut self.context.state[self.context.num_labels as usize * t..];
            // Loop over the attributes attached to the item
            for attr in item {
                // Access the list of state features associated with the attribute
                let id = attr.id;
                let attr_ref = self.model.attr_ref(id)?;
                // A scale usually represents the attribute frequency in the item
                let value = attr.value;
                // Loop over the state features associated with the attribue
                for fid in attr_ref.feature_ids {
                    let feature = self.model.feature(fid)?;
                    if self.randomness == 0.0 {
                        state[feature.target as usize] += feature.weight * value;
                    } else {
                        state[feature.target as usize] += feature.weight
                            * value
                            * thread_rng()
                                .gen_range((1.0 - self.randomness)..(1.0 + self.randomness));
                    }
                }
            }
        }
        Ok(())
    }

    fn viterbi(&mut self) -> (Vec<u32>, f64) {
        self.context.viterbi()
    }
}
