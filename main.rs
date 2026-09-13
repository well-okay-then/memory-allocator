use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};

const CLASSES: [i64; 8] = [16, 32, 64, 128, 256, 512, 1024, 2048];
const TOTAL_HEAP: i64 = 32768;

fn class_for(size: i64) -> Option<usize> {
    CLASSES.iter().position(|&c| c >= size)
}

struct Allocator {
    bump_offset: i64,
    // One free list (LIFO) and live-count per size class.
    free_lists: [Vec<i64>; 8],
    alloc_counts: [i64; 8],
    // Every address ever handed out remembers which class it belongs to,
    // so FREE can find the right free list without the caller telling us.
    class_of_addr: HashMap<i64, usize>,
    allocated: HashSet<i64>,
}

impl Allocator {
    fn new() -> Self {
        Allocator {
            bump_offset: 0,
            free_lists: Default::default(),
            alloc_counts: [0; 8],
            class_of_addr: HashMap::new(),
            allocated: HashSet::new(),
        }
    }

    // Reuse a freed block from the class's own free list before growing
    // the bump region, so freed space is never wasted while it's
    // available.
    fn alloc(&mut self, size: i64) -> Option<i64> {
        let idx = class_for(size.max(1))?;
        let class_size = CLASSES[idx];

        let addr = if let Some(addr) = self.free_lists[idx].pop() {
            addr
        } else {
            if self.bump_offset + class_size > TOTAL_HEAP {
                return None;
            }
            let addr = self.bump_offset;
            self.bump_offset += class_size;
            addr
        };

        self.class_of_addr.insert(addr, idx);
        self.allocated.insert(addr);
        self.alloc_counts[idx] += 1;
        Some(addr)
    }

    fn free(&mut self, addr: i64) -> bool {
        if !self.allocated.remove(&addr) {
            return false;
        }
        let idx = self.class_of_addr[&addr];
        self.alloc_counts[idx] -= 1;
        self.free_lists[idx].push(addr);
        true
    }

    fn print_stats(&self) {
        for i in 0..CLASSES.len() {
            println!(
                "class={} alloc={} free={}",
                CLASSES[i],
                self.alloc_counts[i],
                self.free_lists[i].len()
            );
        }
        println!("bump={}/{}", self.bump_offset, TOTAL_HEAP);
    }
}

fn main() {
    let stdin = io::stdin();
    let mut alloc = Allocator::new();

    for line in stdin.lock().lines() {
        let l = line.unwrap();
        if l.is_empty() {
            continue;
        }

        let spl: Vec<&str> = l.split(' ').collect();

        match spl[0] {
            "ALLOC" => {
                let size = spl[1].parse::<i64>().unwrap();
                match alloc.alloc(size) {
                    Some(addr) => println!("{}", addr),
                    None => println!("OOM"),
                }
            }
            "FREE" => {
                let addr = spl[1].parse::<i64>().unwrap();
                if alloc.free(addr) {
                    println!("OK");
                } else {
                    println!("BAD");
                }
            }
            "STATS" => {
                alloc.print_stats();
            }
            _ => println!("POOP"),
        }
    }
}
