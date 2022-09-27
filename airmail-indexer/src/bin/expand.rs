use std::fs::File;

use airmail_indexer::common::process_osm;
use clap::Parser;
use fst::MapBuilder;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The OSM extract to expand.
    #[clap(long, value_parser)]
    osm: String,
}

fn save_fst(strs: Vec<String>, path: &str) {
    println!("Writing {} strings to FST: {}", strs.len(), path);
    let mut vec: Vec<String> = strs;
    vec.sort();
    let f = File::create(path).unwrap();
    let mut builder = MapBuilder::new(f).unwrap();
    for (i, s) in vec.iter().enumerate() {
        builder.insert(s, i as u64).unwrap();
    }
    builder.finish().unwrap();
}

fn main() {
    let args = Args::parse();
    println!("Processing {}", args.osm);
    let contents = process_osm(args.osm);
    // Save smallest to biggest, because once we save an FST we can drop all of the data in it, and saving an FST takes precious RAM.
    save_fst(contents.countries, "countries.fst");
    save_fst(contents.regions, "regions.fst");
    save_fst(contents.localities, "localities.fst");
    save_fst(contents.neighborhoods, "neighborhoods.fst");
    save_fst(contents.roads, "roads.fst");
    save_fst(contents.house_numbers, "house_numbers.fst");
}
