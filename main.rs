use std::collections::HashMap;
use std::io::{self, BufRead};

struct Slab {
    used: Vec<bool>,
    free_count: i32,
}

impl Slab {
    fn new(cap: i32) -> Self {
        Slab {
            used: vec![false; cap as usize],
            free_count: cap,
        }
    }
}

#[allow(dead_code)]
struct Cache {
    obj_size: i64,
    objs_per_slab: i32,
    slabs: Vec<Slab>,
    objs_alloc: i64,
}

impl Cache {
    fn new(obj_size: i64, objs_per_slab: i32) -> Self {
        Cache {
            obj_size,
            objs_per_slab,
            slabs: Vec::new(),
            objs_alloc: 0,
        }
    }

    // Reuse the first slab (in creation order) that has room before
    // minting a new one, and within a slab always take the lowest free
    // object index — a deterministic, easy-to-reason-about placement.
    fn alloc(&mut self) -> (i32, i32) {
        for (i, slab) in self.slabs.iter_mut().enumerate() {
            if slab.free_count > 0 {
                let j = slab.used.iter().position(|&u| !u).unwrap();
                slab.used[j] = true;
                slab.free_count -= 1;
                self.objs_alloc += 1;
                return (i as i32, j as i32);
            }
        }

        let mut slab = Slab::new(self.objs_per_slab);
        slab.used[0] = true;
        slab.free_count -= 1;
        self.slabs.push(slab);
        self.objs_alloc += 1;
        ((self.slabs.len() - 1) as i32, 0)
    }

    fn free(&mut self, slab_idx: i32, obj_idx: i32) -> bool {
        if slab_idx < 0 || slab_idx as usize >= self.slabs.len() {
            return false;
        }
        let slab = &mut self.slabs[slab_idx as usize];
        if obj_idx < 0 || obj_idx as usize >= slab.used.len() {
            return false;
        }
        if !slab.used[obj_idx as usize] {
            return false;
        }
        slab.used[obj_idx as usize] = false;
        slab.free_count += 1;
        self.objs_alloc -= 1;
        true
    }

    // A slab is empty (no objects in use), full (no room left), or
    // partial (some of each) — the three buckets a slab cache reports so
    // callers can see how much reclaimable memory is sitting idle.
    fn stats(&self) -> String {
        let mut empty = 0;
        let mut partial = 0;
        let mut full = 0;
        for slab in &self.slabs {
            let used = self.objs_per_slab - slab.free_count;
            if used == 0 {
                empty += 1;
            } else if used == self.objs_per_slab {
                full += 1;
            } else {
                partial += 1;
            }
        }
        format!(
            "objs_alloc={} slabs={} empty={} partial={} full={}",
            self.objs_alloc,
            self.slabs.len(),
            empty,
            partial,
            full
        )
    }
}

fn main() {
    let stdin = io::stdin();
    let mut caches: HashMap<String, Cache> = HashMap::new();

    for line in stdin.lock().lines() {
        let l = line.unwrap();
        if l.is_empty() {
            continue;
        }

        let spl: Vec<&str> = l.split(' ').collect();

        match spl[0] {
            "CACHE_CREATE" => {
                let name = spl[1].to_string();
                let obj_size = spl[2].parse::<i64>().unwrap();
                let objs_per_slab = spl[3].parse::<i32>().unwrap();
                caches.insert(name, Cache::new(obj_size, objs_per_slab));
                println!("OK");
            }
            "ALLOC" => {
                let name = spl[1];
                match caches.get_mut(name) {
                    Some(cache) => {
                        let (slab_idx, obj_idx) = cache.alloc();
                        println!("{}:{}", slab_idx, obj_idx);
                    }
                    None => println!("BAD"),
                }
            }
            "FREE" => {
                let name = spl[1];
                let slab_idx = spl[2].parse::<i32>().unwrap();
                let obj_idx = spl[3].parse::<i32>().unwrap();
                match caches.get_mut(name) {
                    Some(cache) => {
                        if cache.free(slab_idx, obj_idx) {
                            println!("OK");
                        } else {
                            println!("BAD");
                        }
                    }
                    None => println!("BAD"),
                }
            }
            "STATS" => {
                let name = spl[1];
                match caches.get(name) {
                    Some(cache) => println!("{}", cache.stats()),
                    None => println!("BAD"),
                }
            }
            _ => println!("POOP"),
        }
    }
}
