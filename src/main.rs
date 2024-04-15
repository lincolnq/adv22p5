use std::collections::HashMap;

// import parse.rs
mod parse;

fn main() {
    println!("Hello, world!");

    // we're going to consume stdin line by line
    //let mut lines = std::io::stdin().lines();
    let mut handle = std::io::stdin().lock();

    // read seeds
    let seeds = parse::parse_seeds(&mut handle);
    println!("Parsed: {:?}", seeds);

    // read map blocks
    let mut map_blocks = Vec::new();
    loop {
        let map_block = parse::parse_map_block(&mut handle);
        if map_block.is_none() {
            break;
        }
        map_blocks.push(map_block.unwrap());
    }
    
    // now run it on all the seeds
    let mut results = HashMap::new();
    for seed in seeds {
        results.insert(seed, forward_pass_all(&map_blocks, seed));
    }
    println!("Results: {:?}", results);

    // show the lowest result
    let min = results.iter().min_by_key(|(_, &v)| v).unwrap();
    println!("Lowest: {:?}", min);

}

// forward pass for a map block: map source to destination across all mappings in that block
fn forward_pass(map_block: &parse::ParsedMapBlock, x: u64) -> u64 {
    let mut matches = map_block.mappings.iter().filter(|(_, srcrs, len)| x >= *srcrs && x < *srcrs + *len);
    if let Some((destrs, srcrs, _)) = matches.next() {
        x - srcrs + destrs
    } else {
        x
    }
}

// forward pass for the entire sequence: map seed to location
fn forward_pass_all(map_blocks: &Vec<parse::ParsedMapBlock>, seed: u64) -> u64 {
    map_blocks.iter().fold(seed, |acc, map_block| forward_pass(map_block, acc))
}