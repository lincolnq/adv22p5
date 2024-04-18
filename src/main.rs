use std::collections::HashMap;
use std::cmp::min;
use std::cmp::max;
use std::ops::Range;

use rangemap::RangeMap;

// import parse.rs
mod parse;
use crate::parse::StageMap;

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

    let mut maps = Vec::new();
    for mb in map_blocks.iter() {
        let rm = collect_map(&mb);
        check_no_overlaps(&rm);
        println!("RangeMap: {:?}, inv: {:?}", rm, invert_map(&rm));
        maps.push(rm);
    }

    let first2 = compose_maps(maps.get(0).unwrap(), maps.get(1).unwrap());
    println!("Composed 1 and 2: {:?}", first2);

    /* How to solve part 2.

    We can start at the final map (humid->location map)
    Collapse it with prior temp->humid map to make Temp->Location Map
    (probably doubles size of maps at each stage)

    Eventually find the minimum location of mapped area.
    *
    
    
    // now run it on all the seeds
    let mut results = HashMap::new();
    for (seed,_) in seeds {
        results.insert(seed, forward_pass_all(&map_blocks, seed));
    }
    println!("Results: {:?}", results);

    // show the lowest result
    let min = results.iter().min_by_key(|(_, &v)| v).unwrap();
    println!("Lowest: {:?}", min);
*/
}



// collect a StageMap's mapping into a RangeMap
fn collect_map(stage_map: &StageMap) -> RangeMap<i64, i64> {
    let mut range_map = RangeMap::new();
    for (udest, usrc, ulen) in &stage_map.mappings {
        let (dest, src, len) = (*udest as i64, *usrc as i64, *ulen as i64);
        let delta = dest - src;
        range_map.insert(src..(src + len), delta);
    }
    range_map
}

// check no overlaps in a RangeMap
fn check_no_overlaps(rm: &RangeMap<i64, i64>) {
    for (range1, _) in rm.iter() {
        // check that the only overlapping range is this one
        let overlapping_ranges = rm.overlapping(range1).map(|x| x.0).collect::<Vec<&Range<i64>>>();
        if overlapping_ranges != vec![range1] {
            // we have an overlap
            println!("Overlapping ranges in RangeMap: {:?}", rm);
        }
    }
}

// invert a rangemap so that instead of mapping 'forward' from source to destination, it maps 'backward' from destination to source
fn invert_map(rm: &RangeMap<i64, i64>) -> RangeMap<i64, i64> {
    let mut result = RangeMap::new();
    for (range, delta) in rm.iter() {
        result.insert((range.start as i64 + delta) as i64..(range.end as i64 + delta) as i64, -delta);
    }
    result
}


fn offset_range(range: Range<i64>, offset: i64) -> Range<i64> {
    (range.start as i64 + offset) as i64..(range.end as i64 + offset) as i64
}


// 


// add_overlapping_range takes a step in merging range1 into the given rangemap by summing overlapping ranges.
// The rangemap must not have any overlapping ranges before this step.
// Returns the remaining range from range1 that has not been merged (e.g. remnant after the first overlap found)
// or None if range1 was fully merged.
// In the result, whenever range start/ends comes from range1, we offset those by `-range1val` in order to place the 
// output map into the x-space
fn add_overlapping_range(result: &mut RangeMap<i64, i64>, range1: Range<i64>, range1val: i64) -> Option<Range<i64>> {

    let ovl2 = result.overlapping(&range1).next();
    if ovl2.is_none() {
        // no overlap
        println!("No overlap for range: {:?}", range1);
        result.insert(offset_range(range1.clone(), -range1val), range1val);
        return None;
    }
    let (range2c, &range2val) = ovl2.unwrap();
    let range2 = range2c.clone();
    result.remove(range2.clone());
    println!("First overlapping range for {:?}: {:?}", range1, range2);

    // handle pre-overlap
    if range1.start < range2.start {
        // map1's range starts first
        let new_range = range1.start - range1val..range2.start;
        println!("Pre-overlap x: {:?} = {}", new_range, range1val);
        result.insert(new_range, range1val);
    } else if range2.start < range1.start {
        // map2's range starts first
        let new_range = range2.start..range1.start - range1val;
        println!("Pre-overlap: {:?} = {}", new_range, range2val);
        result.insert(new_range, range2val);
    } // else they start at the same time, no need for pre-overlap

    // now we need to do the overlapping section
    let start_of_overlap = max(range1.start, range2.start);
    let end_of_overlap = min(range1.end, range2.end);
    println!("Overlap: {:?} = {}", start_of_overlap..end_of_overlap, range1val + range2val);
    result.insert(offset_range(start_of_overlap..end_of_overlap, -range1val), range1val + range2val);

    // handle post-overlap
    if range2.end > range1.end {
        // map2's range is longer, so add rest of range2
        let new_range = range1.end..range2.end;
        // this one doesn't get offset, it's essentially the remnant which was splitout
        result.insert(new_range, range2val);

        // no range1 remnant
        None
    } else if range1.end > range2.end {
        // range1 is longer, so return the remnant
        Some(range2.end..range1.end)
    } else {
        // they end at the same time, no remnant
        None
    }
}


// compose_maps takes two RangeMaps and composes them into a new RangeMap by calling a zipper function
// on each subrange of the two maps. 
// We automatically divide ranges when the two maps collide.
// The left and right arguments to the zipper function are the current range and value of the two maps, respectively. 
// The zipper function should return a new range and value to insert into the result map.

type Zipper = fn(Option(&Range<i64>, &i64), Option(&Range<i64>, &i64)) -> (Range<i64>, i64)

fn compose_maps_new(zipper: Zipper, map1: &RangeMap<i64, i64>, map2: &RangeMap<i64, i64>) -> RangeMap<i64, i64> {
    // walk both maps in parallel -- need two iterators, and we'll create new ranges in our output map
    let mut result = RangeMap::new();

    let mut iter1 = map1.iter();
    let mut iter2 = map2.iter();

    let mut next1 = iter1.next();
    let mut next2 = iter2.next();

    let mut nextpt = None;

    loop {
        if next1.is_none() && next2.is_none() {
            // both maps are done
            break;
        }

        if next1.is_none() {
            for (range2, delta2) in iter2 {
                // map1 is done, so just zipper all of map2
                result.insert(zipper(None, Some(range2, delta2)));
            }
            break;
        } else if next2.is_none() {
            for (range1, delta1) in iter1 {
                // map2 is done, so just add all of map1
                result.insert(zipper(Some(range1, delta1), None));
            }
            break;
        }

        let (range1, delta1) = next1.unwrap();
        let (range2, delta2) = next2.unwrap();


        if range1.start < range2.start { 
            if range1.end < range2.start {
                // range1 ends before range2 starts
                result.insert(zipper(Some(range1, delta1), None));
                next1 = iter1.next();
            } else {
                // range1 overlaps range2 - do two zippers
                let newrange = range1.start..range2.start;
                result.insert(zipper(Some(newrange, delta1), Some(range2, delta2));
                next1 = Some(range2.start..range1.end);
            }
            // create new range from range1 until either range2 starts 
            let newrange = range1.start..range2.start;
            result.insert(zipper(Some(newrange, delta1), None));
        }

        break;
    
    }
    result
}



// compose_maps takes two RangeMaps (x-y and y-z) and composes them into a new RangeMap. This is done by
// converting all our map1 ys (outputs) into the space of map2's ys (inputs) in order to find overlaps, but 
// we then back it into x-space

fn compose_maps(map1: &RangeMap<i64, i64>, map2: &RangeMap<i64, i64>) -> RangeMap<i64, i64> {

    // first invert map1
    //let map1_inv = invert_map(map1);

    



    // we'll make a new version of map2. For each map1 range, we need to convert it into map2
    // space, and then add its deltas into the result.
    let mut result = map2.clone();
    
    println!("Compose_maps called with {:?}", result);


    for (range1, delta) in map1.iter() {

        // convert the x-space of this range into its y-space by adding delta
        let newrange = offset_range(range1.clone(), *delta);

        //let range1_map1_start = range2.start as i64 + map1_inv.get(&range2.start).unwrap_or(&0);
        //let range2_len = (range2.end - range2.start) as i64;
        //let mut range2_map1 = Some(range2_map1_start as u64..(range2_map1_start + range2_len) as u64);

        println!("Merging newrange: {:?} (was: {:?}) into result", newrange, range1);
        let mut mayberange = Some(newrange);

        while let Some(range) = mayberange {
            mayberange = add_overlapping_range(&mut result, range, *delta);
            println!("Middle result is {:?}", result);

        }

        println!("Merged: result is {:?}", result);

    }
    result
}

/*
    Collapse two maps e.g. temp->humid map, humid->location, to make Temp->Location Map
    (probably doubles size of maps at each stage)

example:

x-to-y map:
y x range
0 69 1
1 0 69

y-to-z map:
z y range
60 56 37
56 93 4

z=56_4 -> y=93_4 -> x=93_4*
z=60_37 -> y=56_37 -> x=55_37

*
fn collapse_maps(map1: &StageMap, map2: &StageMap) -> StageMap {

    let mut mappings = Vec::new();
    for (yst, xst, range1) in map1.mappings {
        for (zst, yst, range2) in map2.mappings {
            
        }
    }

    StageMap {
        from: map1.from.clone(),
        to: map2.to.clone(),
        mappings: map1.mappings.iter().flat_map(|(seed1, soil1, prob1)| {
            map2.mappings.iter().map(move |(seed2, soil2, prob2)| {
                (seed1, soil2, prob1 * prob2)
            })
        }).collect()
    }
}

type Stage = BTreeMap<u64, u64>;

fn find_in_stage(stage: &Stage, x: u64) -> Option<u64> {

    let next = stage.range(x..).next()?
    
}*/