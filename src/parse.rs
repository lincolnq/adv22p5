use std::io::BufRead;

// parse the seeds line from a filehandle
// looks like this: 
//    seeds: 79 14 55 13
// always 1 line, but should consume 2 lines
pub fn parse_seeds<R: BufRead>(reader: &mut R) -> Vec<(u64, u64)> {
    let mut seeds = Vec::new();
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let mut parts = line.split_whitespace();
    assert_eq!(parts.next().unwrap(), "seeds:");
    for part in parts {
        seeds.push(part.parse().unwrap());
    }
    // consume the newline
    reader.read_line(&mut line).unwrap();
    seeds.chunks(2).map(|chunk| (chunk[0], chunk[1])).collect()
}

// parse a map block from the filehandle
// looks like this:
//   seed-to-soil map:
//   50 98 2
//   52 50 48
// ends in a blank line
// we need the "seed-to-soil" string parsed into a tuple of (seed, soil)

// first define a result datatype:
#[derive(Debug)]
pub struct StageMap {
    pub from: String,
    pub to: String,
    pub mappings: Vec<(u64, u64, u64)>,
}

// now the parser
pub fn parse_map_block<R: BufRead>(reader: &mut R) -> Option<StageMap> {
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    if line.trim().is_empty() {
        return None;
    }

    let fromto = line.split_whitespace().next().unwrap().split("-to-").collect::<Vec<&str>>();
    let from = String::from(fromto[0]);
    let to = String::from(fromto[1]);

    // ok, now let's read the mappings
    let mut mappings = Vec::new();

    loop {
        line.clear();
        reader.read_line(&mut line).unwrap();
        if line.trim().is_empty() {
            break;
        }
        let mut parts = line.split_whitespace();
        let seed = parts.next().unwrap().parse().unwrap();
        let soil = parts.next().unwrap().parse().unwrap();
        let prob = parts.next().unwrap().parse().unwrap();
        mappings.push((seed, soil, prob));
    }

    Some(StageMap
     { from: from, to: to, mappings: mappings })
}