use std::collections::HashMap;
use std::io::{self, BufRead};

// How many objects a refill from the central allocator hands to a thread's
// local cache. The requesting call consumes one of them immediately, so an
// empty-cache alloc leaves ALLOC_BATCH - 1 objects cached locally.
const ALLOC_BATCH: i32 = 4;

// How many freed objects a thread's local cache holds before it's full.
// The free that would push it past this flushes the whole batch back to
// the central allocator instead of growing the cache further.
const FREE_CAPACITY: i32 = 5;

fn main() {
    let stdin = io::stdin();
    // One local-cache count per (thread_id, size) pair.
    let mut local: HashMap<(i64, i64), i32> = HashMap::new();

    for line in stdin.lock().lines() {
        let l = line.unwrap();
        if l.is_empty() {
            continue;
        }

        let spl: Vec<&str> = l.split(' ').collect();

        match spl[0] {
            "ALLOC" => {
                let thread = spl[1].parse::<i64>().unwrap();
                let size = spl[2].parse::<i64>().unwrap();
                let count = local.entry((thread, size)).or_insert(0);
                if *count > 0 {
                    *count -= 1;
                    println!("local");
                } else {
                    *count = ALLOC_BATCH - 1;
                    println!("central");
                }
            }
            "FREE" => {
                let thread = spl[1].parse::<i64>().unwrap();
                let size = spl[2].parse::<i64>().unwrap();
                let count = local.entry((thread, size)).or_insert(0);
                if *count < FREE_CAPACITY {
                    *count += 1;
                    println!("local");
                } else {
                    *count = 0;
                    println!("flush");
                }
            }
            "STATS" => {
                let thread = spl[1].parse::<i64>().unwrap();
                let mut classes: Vec<(i64, i32)> = local
                    .iter()
                    .filter(|&(&(t, _), _)| t == thread)
                    .map(|(&(_, size), &count)| (size, count))
                    .collect();
                classes.sort_by_key(|&(size, _)| size);
                for (size, count) in classes {
                    println!("class={}:{}", size, count);
                }
            }
            _ => println!("POOP"),
        }
    }
}
