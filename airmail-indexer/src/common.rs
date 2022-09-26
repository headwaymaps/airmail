use std::{
    collections::HashSet,
    fs::File,
    hash::BuildHasherDefault,
    io::BufReader,
    sync::mpsc::{self, Receiver, SyncSender},
    time::Duration,
};

use deunicode::deunicode;
use highway::HighwayHasher;
use osmpbf::{Element, ElementReader};

#[derive(Debug, Clone)]
pub struct ElemContents {
    pub name: Option<String>,
    pub house_number: Option<String>,
    pub unit_number: Option<String>,
    pub road: Option<String>,
    pub neighborhood: Option<String>,
    pub locality: Option<String>,
    pub postcode: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractContents {
    pub house_numbers: Vec<String>,
    pub roads: Vec<String>,
    pub neighborhoods: Vec<String>,
    pub localities: Vec<String>,
    pub regions: Vec<String>,
    pub countries: Vec<String>,
}

pub(crate) fn get_reader(filename: String) -> ElementReader<BufReader<File>> {
    ElementReader::from_path(filename).expect("Need an OSM extract")
}

pub(crate) fn process_nodes_and_ways(element: Element) -> Option<ElemContents> {
    let tags = match element {
        Element::Node(node) => node.tags(),
        Element::DenseNode(_dense_node) => return None,
        Element::Way(way) => way.tags(),
        Element::Relation(_relation) => return None,
    };
    let mut names = vec![];
    let mut house_numbers = vec![];
    let mut unit_numbers = vec![];
    let mut roads = vec![];
    let mut neighborhoods = vec![];
    let mut localities = vec![];
    let mut postcodes = vec![];
    let mut regions = vec![];
    let mut countries = vec![];
    for (key, value) in tags {
        let value = deunicode(value).to_ascii_lowercase();
        match key {
            "addr:name" => names.push(value),
            "addr:housenumber" => house_numbers.push(value),
            "addr:unit" => unit_numbers.push(value),
            "addr:street" => roads.push(value),
            "addr:suburb" => neighborhoods.push(value),
            "addr:neighbourhood" => neighborhoods.push(value),
            "addr:subdistrict" => neighborhoods.push(value),
            "addr:city" => localities.push(value),
            "addr:municipality" => localities.push(value),
            "addr:district" => localities.push(value),
            "addr:postcode" => postcodes.push(value),
            "addr:state" => regions.push(value),
            "addr:province" => regions.push(value),
            "addr:country" => countries.push(value),
            _ => {}
        }
    }
    Some(ElemContents {
        name: if let Some(val) = names.get(0) {
            Some(val.clone())
        } else {
            None
        },
        locality: if let Some(val) = localities.get(0) {
            Some(val.clone())
        } else {
            None
        },
        house_number: if let Some(val) = house_numbers.get(0) {
            Some(val.clone())
        } else {
            None
        },
        road: if let Some(val) = roads.get(0) {
            Some(val.clone())
        } else {
            None
        },
        unit_number: if let Some(val) = unit_numbers.get(0) {
            Some(val.clone())
        } else {
            None
        },
        neighborhood: if let Some(val) = neighborhoods.get(0) {
            Some(val.clone())
        } else {
            None
        },
        postcode: if let Some(val) = postcodes.get(0) {
            Some(val.clone())
        } else {
            None
        },
        region: if let Some(val) = regions.get(0) {
            Some(val.clone())
        } else {
            None
        },
        country: if let Some(val) = countries.get(0) {
            Some(val.clone())
        } else {
            None
        },
    })
}

pub(crate) fn process_dense_nodes(element: Element) -> Option<ElemContents> {
    let tags = match element {
        Element::Node(_node) => return None,
        Element::DenseNode(dense_node) => dense_node.tags(),
        Element::Way(_way) => return None,
        Element::Relation(_relation) => return None,
    };
    let mut names = vec![];
    let mut house_numbers = vec![];
    let mut unit_numbers = vec![];
    let mut roads = vec![];
    let mut neighborhoods = vec![];
    let mut localities = vec![];
    let mut postcodes = vec![];
    let mut regions = vec![];
    let mut countries = vec![];
    for (key, value) in tags {
        let value = deunicode(value).to_ascii_lowercase();
        match key {
            "addr:name" => names.push(value),
            "addr:housenumber" => house_numbers.push(value),
            "addr:unit" => unit_numbers.push(value),
            "addr:street" => roads.push(value),
            "addr:suburb" => neighborhoods.push(value),
            "addr:neighbourhood" => neighborhoods.push(value),
            "addr:subdistrict" => neighborhoods.push(value),
            "addr:city" => localities.push(value),
            "addr:municipality" => localities.push(value),
            "addr:district" => localities.push(value),
            "addr:postcode" => postcodes.push(value),
            "addr:state" => regions.push(value),
            "addr:province" => regions.push(value),
            "addr:country" => countries.push(value),
            _ => {}
        }
    }
    Some(ElemContents {
        name: if let Some(val) = names.get(0) {
            Some(val.clone())
        } else {
            None
        },
        locality: if let Some(val) = localities.get(0) {
            Some(val.clone())
        } else {
            None
        },
        house_number: if let Some(val) = house_numbers.get(0) {
            Some(val.clone())
        } else {
            None
        },
        road: if let Some(val) = roads.get(0) {
            Some(val.clone())
        } else {
            None
        },
        unit_number: if let Some(val) = unit_numbers.get(0) {
            Some(val.clone())
        } else {
            None
        },
        neighborhood: if let Some(val) = neighborhoods.get(0) {
            Some(val.clone())
        } else {
            None
        },
        postcode: if let Some(val) = postcodes.get(0) {
            Some(val.clone())
        } else {
            None
        },
        region: if let Some(val) = regions.get(0) {
            Some(val.clone())
        } else {
            None
        },
        country: if let Some(val) = countries.get(0) {
            Some(val.clone())
        } else {
            None
        },
    })
}

pub(crate) fn process_relations(element: Element) -> Option<ElemContents> {
    let tags = match element {
        Element::Node(_node) => return None,
        Element::DenseNode(_dense_node) => return None,
        Element::Way(_way) => return None,
        Element::Relation(relation) => relation.tags(),
    };
    let mut names = vec![];
    let mut house_numbers = vec![];
    let mut unit_numbers = vec![];
    let mut roads = vec![];
    let mut neighborhoods = vec![];
    let mut localities = vec![];
    let mut postcodes = vec![];
    let mut regions = vec![];
    let mut countries = vec![];
    for (key, value) in tags {
        let value = deunicode(value).to_ascii_lowercase();
        match key {
            "addr:name" => names.push(value),
            "addr:housenumber" => house_numbers.push(value),
            "addr:unit" => unit_numbers.push(value),
            "addr:street" => roads.push(value),
            "addr:suburb" => neighborhoods.push(value),
            "addr:neighbourhood" => neighborhoods.push(value),
            "addr:subdistrict" => neighborhoods.push(value),
            "addr:city" => localities.push(value),
            "addr:municipality" => localities.push(value),
            "addr:district" => localities.push(value),
            "addr:postcode" => postcodes.push(value),
            "addr:state" => regions.push(value),
            "addr:province" => regions.push(value),
            "addr:country" => countries.push(value),
            _ => {}
        }
    }
    Some(ElemContents {
        name: if let Some(val) = names.get(0) {
            Some(val.clone())
        } else {
            None
        },
        locality: if let Some(val) = localities.get(0) {
            Some(val.clone())
        } else {
            None
        },
        house_number: if let Some(val) = house_numbers.get(0) {
            Some(val.clone())
        } else {
            None
        },
        road: if let Some(val) = roads.get(0) {
            Some(val.clone())
        } else {
            None
        },
        unit_number: if let Some(val) = unit_numbers.get(0) {
            Some(val.clone())
        } else {
            None
        },
        neighborhood: if let Some(val) = neighborhoods.get(0) {
            Some(val.clone())
        } else {
            None
        },
        postcode: if let Some(val) = postcodes.get(0) {
            Some(val.clone())
        } else {
            None
        },
        region: if let Some(val) = regions.get(0) {
            Some(val.clone())
        } else {
            None
        },
        country: if let Some(val) = countries.get(0) {
            Some(val.clone())
        } else {
            None
        },
    })
}

pub fn process_osm(filename: String) -> ExtractContents {
    crossbeam::scope(move |scope| {
        let reader = get_reader(filename);
        let (sender, receiver): (
            SyncSender<Option<ElemContents>>,
            Receiver<Option<ElemContents>>,
        ) = mpsc::sync_channel(100000);
        let (list_sender, list_receiver) = mpsc::sync_channel(1024);

        let handle = scope.spawn(move |_| {
            let mut house_numbers =
                HashSet::with_hasher(BuildHasherDefault::<HighwayHasher>::default());
            let mut roads = HashSet::with_hasher(BuildHasherDefault::<HighwayHasher>::default());
            let mut neighborhoods =
                HashSet::with_hasher(BuildHasherDefault::<HighwayHasher>::default());
            let mut localities =
                HashSet::with_hasher(BuildHasherDefault::<HighwayHasher>::default());
            let mut regions = HashSet::with_hasher(BuildHasherDefault::<HighwayHasher>::default());
            let mut countries =
                HashSet::with_hasher(BuildHasherDefault::<HighwayHasher>::default());
            loop {
                match receiver.recv_timeout(Duration::from_secs(5)).unwrap() {
                    Some(elem) => {
                        house_numbers.extend(elem.house_number);
                        roads.extend(elem.road);
                        neighborhoods.extend(elem.neighborhood);
                        localities.extend(elem.locality);
                        regions.extend(elem.region);
                        countries.extend(elem.country);
                    }
                    None => {
                        list_sender
                            .send(ExtractContents {
                                house_numbers: house_numbers.into_iter().collect(),
                                roads: roads.into_iter().collect(),
                                neighborhoods: neighborhoods.into_iter().collect(),
                                localities: localities.into_iter().collect(),
                                regions: regions.into_iter().collect(),
                                countries: countries.into_iter().collect(),
                            })
                            .unwrap();
                        break;
                    }
                }
            }
        });
        reader
            .par_map_reduce(
                |element| {
                    if let Some(contents) = process_nodes_and_ways(element.clone()) {
                        sender
                            .clone()
                            .send(Some(contents))
                            .expect("Send must succeed");
                    } else if let Some(contents) = process_dense_nodes(element.clone()) {
                        sender
                            .clone()
                            .send(Some(contents))
                            .expect("Send must succeed");
                    } else if let Some(contents) = process_relations(element.clone()) {
                        sender
                            .clone()
                            .send(Some(contents))
                            .expect("Send must succeed");
                    }
                },
                || (),
                |_a, _b| (),
            )
            .expect("Need for_each to work!");
        sender.send(None).expect("need send to succeed");
        handle.join().unwrap();
        list_receiver.recv().expect("need list")
    })
    .unwrap()
}
