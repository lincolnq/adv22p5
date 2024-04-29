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

    let example = 79;
    let nstages = 7;

    let mut value = example;
    // apply example to the first nstages in order
    for i in 0..nstages {
        let delta = maps[i].get(&value).unwrap_or(&0);
        println!("Stage: {:?}-to-{:?}: {:?} -> {:?}", map_blocks[i].from, map_blocks[i].to, value, value + delta);
        value += delta;
    }
    
    // now combine stages and try again
    let combined = maps.into_iter().take(nstages).reduce(combine_stages).unwrap();
    println!("Combined {:?} stages: {:?}", nstages, combined);

    println!("Combined result: {:?}", combined.get(&example).unwrap_or(&0) + example);
    
    
    // now solve part 2 - first construct the seed map
    let mut seed_map = RangeMap::new();
    for (seed, count) in seeds {
        seed_map.insert(seed as i64..(seed + count) as i64, 0);
    }

    println!("Seed map: {:?}", seed_map);

    // and then intersect it with the combined map.
    let final_map = compose_maps_new(|r, x1, x2| {
        // where the seed map has nothing, we just return None
        if x1.is_none() {
            return None;
        }

        // otherwise use r's start plus the delta (x2) as the result
        Some((r.clone(), r.start + x2.unwrap_or(&0)))

    }, &seed_map, &combined);
    
    println!("Final map: {:?}", final_map);
    // find the minimum value in the final map
    let result = final_map.iter().map(|(_, v)| v).min();
    println!("Min: {:?}", result);
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

// implement RangeMap.pop_first()
// returns clones of the range and value
fn pop_first<K, V>(rm: &mut RangeMap<K, V>) -> Option<(Range<K>, V)>
where K: Ord + Clone, V: Eq + Clone 
{
    let (range, value) = rm.first_range_value()?;
    let rc = range.clone();
    let vc = value.clone();
    rm.remove(rc.clone());
    Some((rc, vc))
}

// implement RangeMap.insert() with an optional pair of (Range, Value)
fn insert<K, V>(rm: &mut RangeMap<K, V>, kv: Option<(Range<K>, V)>)
where K: Ord + Clone, V: Eq + Clone 
{
    if let Some((k, v)) = kv {
        rm.insert(k, v);
    }
}

// implement intersection on Range<K>
fn intersection<K>(
    r1: &Range<K>, r2: &Range<K>
) -> Option<Range<K>> 
where K: Ord + Clone
{
    let start = max(r1.start.clone(), r2.start.clone());
    let end = min(r1.end.clone(), r2.end.clone());
    if start < end {
        Some(start..end)
    } else {
        None
    }
}

// compose_maps_new takes two RangeMaps and composes them into a new RangeMap by calling a zipper function
// on each subrange of the two maps. 
// We automatically divide ranges when the two maps collide.
// The left and right arguments to the zipper function are the current range and value of the two maps, respectively. 
// The zipper function should return a new range and value to insert into the result map, or None
// if you don't want to insert anything.

type Zipper<K,V> = fn(Range<K>, Option<&V>, Option<&V>) -> Option<(Range<K>, V)>;

fn compose_maps_new<K, V>(
    zipper: Zipper<K, V>, 
    map1: &RangeMap<K, V>,
    map2: &RangeMap<K, V>)
     -> RangeMap<K, V> 
    where K: Ord + Clone, V: Eq + Clone 
{
    let mut result = RangeMap::new();

    // Our algo requires mutating both input maps, so we just clone them.
    let mut m1 = map1.clone();
    let mut m2 = map2.clone();

    //println!("Starting CMN {:?} + {:?}", m1, m2);


    // Consume map 1 first
    while let Some(mut r1) = pop_first(&mut m1) {
        // r1 is (range, value)

        //println!("Examining M1: {:?}", r1);

        
        // try to intersect
        while let Some(r2) = m2.overlapping(r1.clone().0).next() {
            let int = intersection(&r1.0, r2.0).unwrap();
            //println!("First overlap M2: {:?} int: {:?}", r2, int);
            let v2 = r2.1.clone();

            // ok, there is an intersection. first yield the pre-intersect
            if r1.0.start < int.start {
                //println!("Pre-intersect: {:?}", int);
                insert(&mut result, zipper(r1.0.start..int.clone().start, Some(&r1.1), None));
            }
            // now yield the intersect
            insert(&mut result, zipper(int.clone(), Some(&r1.1), Some(&v2)));
            // and remove the intersection from m2
            m2.remove(int.clone());
            // now our r1 'remnant' is the remainder of r1, to be intersected
            // with the remainder of m2. Note that thsi could be a zero length
            // remnant but will not be negative.
            r1 = ((int.end..r1.0.end), r1.1);
        }

        // ok, no more overlapping sections for r1, thus we can insert it
        // (if non-zero)
        if r1.0.start < r1.0.end {
            insert(&mut result, zipper(r1.0.start..r1.0.end, Some(&r1.1), None));
        }
    }

    // ok, no more m1. insert all remaining items from m2 as non-overlapping regions
    for (r, r2val) in m2.iter() {
        insert(&mut result, zipper(r.clone(), None, Some(r2val)))
    }

    result
}

// combine_stages takes two RangeMaps (x-y and y-z) and composes them into a new RangeMap. This is done by
// converting all our map1 ys (outputs) into the space of map2's ys (inputs) in order to find overlaps, but 
// we then back it into x-space

// EXAMPLE
//seed-to-soil map:
//98..100 -> 50..52 [-48] len=2
//50..98 -> 52..100 [+2] len=48

// INVERTED: {50..52:+48, 52..100:-2}

//soil-to-fertilizer map:
//15..52 -> 0..37 [-15] len=37
//52..54 -> 37..39 [-15] len=2
//0..15 -> 39..54 [+39] len=15

// RESULT
//seed-to-fertilizer map:
//0..15 -> 39..54 [+39] len=15
//15..50 -> 0..35 [-15] len=35
//50..52 -> 37..39 [-13] len=2
//52..98 -> 54..100 [+2] len=46
//98..100 -> 35..37 [-63] len=2


fn combine_stages(map1: RangeMap<i64, i64>, map2: RangeMap<i64, i64>) -> RangeMap<i64, i64> {
    
    let map1_inv = invert_map(&map1);

    compose_maps_new(|r, x1, x2| {
        // the new delta is just the sum of whichever are present, but negate
        // the delta of x1 since it was from the inverse map
        let delta = -x1.unwrap_or(&0) + x2.unwrap_or(&0);

        // the range given is in y-space, needs to be in x-space, so
        // map it back
        let range = offset_range(r, *x1.unwrap_or(&0));
        Some((range, delta))
    }, &map1_inv, &map2)
}
