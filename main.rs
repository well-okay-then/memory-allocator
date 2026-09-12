use std::collections::HashMap;
use std::io::{self, BufRead};

// Smallest k such that 2^k >= size (size 0 or 1 both need order 0).
fn order_for(size: i64) -> i32 {
    let mut k: i32 = 0;
    while (1i64 << k) < size {
        k += 1;
    }
    k
}

struct Buddy {
    max_order: i32,
    free_lists: Vec<Vec<i64>>,
    used: HashMap<i64, i32>,
}

impl Buddy {
    fn new() -> Self {
        Buddy {
            max_order: -1,
            free_lists: Vec::new(),
            used: HashMap::new(),
        }
    }

    fn init(&mut self, order: i32) {
        self.max_order = order;
        self.free_lists = vec![Vec::new(); (order + 1) as usize];
        self.used.clear();
        self.free_lists[order as usize].push(0);
    }

    // Find the smallest free block whose order can satisfy `req`, then
    // repeatedly split it in half — stashing the unused buddy at each
    // level's free list — until it's exactly the requested order.
    fn alloc(&mut self, size: i64) -> Option<i64> {
        let req = order_for(size.max(1));
        if req > self.max_order {
            return None;
        }

        let mut f = req;
        while f <= self.max_order && self.free_lists[f as usize].is_empty() {
            f += 1;
        }
        if f > self.max_order {
            return None;
        }

        let idx = self.free_lists[f as usize]
            .iter()
            .enumerate()
            .min_by_key(|&(_, &a)| a)
            .map(|(i, _)| i)
            .unwrap();
        let addr = self.free_lists[f as usize].remove(idx);

        let mut order = f;
        while order > req {
            order -= 1;
            let buddy_addr = addr + (1i64 << order);
            self.free_lists[order as usize].push(buddy_addr);
        }

        self.used.insert(addr, req);
        Some(addr)
    }

    // Coalesce with the buddy while it's free, walking up toward max_order.
    fn free(&mut self, addr: i64) -> bool {
        let order = match self.used.remove(&addr) {
            Some(o) => o,
            None => return false,
        };

        let mut cur_addr = addr;
        let mut cur_order = order;
        while cur_order < self.max_order {
            let buddy_addr = cur_addr ^ (1i64 << cur_order);
            let list = &mut self.free_lists[cur_order as usize];
            match list.iter().position(|&a| a == buddy_addr) {
                Some(pos) => {
                    list.remove(pos);
                    cur_addr = cur_addr.min(buddy_addr);
                    cur_order += 1;
                }
                None => break,
            }
        }

        self.free_lists[cur_order as usize].push(cur_addr);
        true
    }

    fn print_freelist(&self, order: i32) {
        if order < 0 || order > self.max_order {
            return;
        }
        let mut addrs = self.free_lists[order as usize].clone();
        addrs.sort();
        for a in addrs {
            println!("{}", a);
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let mut buddy = Buddy::new();

    for line in stdin.lock().lines() {
        let l = line.unwrap();
        if l.is_empty() {
            continue;
        }

        let spl: Vec<&str> = l.split(' ').collect();

        match spl[0] {
            "INIT" => {
                let order = spl[1].parse::<i32>().unwrap();
                buddy.init(order);
                println!("OK");
            }
            "ORDER" => {
                let size = spl[1].parse::<i64>().unwrap();
                println!("{}", order_for(size.max(1)));
            }
            "ALLOC" => {
                let size = spl[1].parse::<i64>().unwrap();
                match buddy.alloc(size) {
                    Some(addr) => println!("{}", addr),
                    None => println!("OOM"),
                }
            }
            "FREE" => {
                let addr = spl[1].parse::<i64>().unwrap();
                if buddy.free(addr) {
                    println!("OK");
                } else {
                    println!("BAD");
                }
            }
            "FREELIST" => {
                let order = spl[1].parse::<i32>().unwrap();
                buddy.print_freelist(order);
            }
            _ => println!("POOP"),
        }
    }
}
