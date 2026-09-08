use std::io::{self, BufRead};

const MIN_SPLIT: i32 = 16;

#[derive(Clone, Copy)]
enum Strategy {
    First,
    Best,
    Worst,
}

struct Block {
    addr: i32,
    size: i32,
    used: bool,
    prev: Option<usize>,
    next: Option<usize>,
}

struct Allocator {
    blocks: Vec<Block>,
    head: Option<usize>,
    strategy: Strategy,
}

impl Allocator {
    fn new() -> Self {
        Allocator {
            blocks: Vec::new(),
            head: None,
            strategy: Strategy::First,
        }
    }

    fn init(&mut self, size: i32, strategy: Strategy) {
        self.blocks.clear();
        self.head = None;
        self.strategy = strategy;
        if size > 0 {
            self.blocks.push(Block {
                addr: 0,
                size,
                used: false,
                prev: None,
                next: None,
            });
            self.head = Some(0);
        }
    }

    // Walk the free list once, picking the winning block per the active
    // placement policy: first fit early-exits, best/worst scan everything.
    fn find_fit(&self, req: i32) -> Option<usize> {
        let mut cur = self.head;
        let mut best: Option<usize> = None;
        while let Some(idx) = cur {
            let blk = &self.blocks[idx];
            if !blk.used && blk.size >= req {
                match self.strategy {
                    Strategy::First => return Some(idx),
                    Strategy::Best => {
                        if best.map_or(true, |b| blk.size < self.blocks[b].size) {
                            best = Some(idx);
                        }
                    }
                    Strategy::Worst => {
                        if best.map_or(true, |b| blk.size > self.blocks[b].size) {
                            best = Some(idx);
                        }
                    }
                }
            }
            cur = blk.next;
        }
        best
    }

    fn alloc(&mut self, req: i32) -> Option<i32> {
        let idx = self.find_fit(req)?;
        let addr = self.blocks[idx].addr;
        let remainder = self.blocks[idx].size - req;
        self.blocks[idx].used = true;

        if remainder >= MIN_SPLIT {
            let old_next = self.blocks[idx].next;
            self.blocks[idx].size = req;

            let new_idx = self.blocks.len();
            self.blocks.push(Block {
                addr: addr + req,
                size: remainder,
                used: false,
                prev: Some(idx),
                next: old_next,
            });
            self.blocks[idx].next = Some(new_idx);
            if let Some(n) = old_next {
                self.blocks[n].prev = Some(new_idx);
            }
        }

        Some(addr)
    }

    fn free(&mut self, addr: i32) -> bool {
        let mut cur = self.head;
        while let Some(idx) = cur {
            if self.blocks[idx].addr == addr {
                if !self.blocks[idx].used {
                    return false;
                }
                self.blocks[idx].used = false;
                self.coalesce(idx);
                return true;
            }
            cur = self.blocks[idx].next;
        }
        false
    }

    // Absorb free neighbors into `idx` (next first, then prev) so runs of
    // adjacent free blocks always collapse to a single list node.
    fn coalesce(&mut self, idx: usize) {
        if let Some(next_idx) = self.blocks[idx].next {
            if !self.blocks[next_idx].used {
                let next_next = self.blocks[next_idx].next;
                self.blocks[idx].size += self.blocks[next_idx].size;
                self.blocks[idx].next = next_next;
                if let Some(n) = next_next {
                    self.blocks[n].prev = Some(idx);
                }
            }
        }

        if let Some(prev_idx) = self.blocks[idx].prev {
            if !self.blocks[prev_idx].used {
                let idx_next = self.blocks[idx].next;
                self.blocks[prev_idx].size += self.blocks[idx].size;
                self.blocks[prev_idx].next = idx_next;
                if let Some(n) = idx_next {
                    self.blocks[n].prev = Some(prev_idx);
                }
            }
        }
    }

    fn print_freelist(&self) {
        let mut cur = self.head;
        while let Some(idx) = cur {
            let b = &self.blocks[idx];
            if !b.used {
                println!("{}:{}", b.addr, b.size);
            }
            cur = b.next;
        }
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
            "INIT" => {
                let size = spl[1].parse::<i32>().unwrap();
                let strategy = match spl.get(2) {
                    Some(&"BEST") => Strategy::Best,
                    Some(&"WORST") => Strategy::Worst,
                    _ => Strategy::First,
                };
                alloc.init(size, strategy);
                println!("OK");
            }
            "ALLOC" => {
                let t = spl[1].parse::<i32>().unwrap();
                match alloc.alloc(t) {
                    Some(addr) => println!("{}", addr),
                    None => println!("OOM"),
                }
            }
            "FREE" => {
                let addr = spl[1].parse::<i32>().unwrap();
                if alloc.free(addr) {
                    println!("OK");
                } else {
                    println!("BAD");
                }
            }
            "FREELIST" => alloc.print_freelist(),
            _ => println!("POOP"),
        }
    }
}
