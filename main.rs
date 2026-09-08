use std::io::{self, BufRead};

const HEAD_SIZE: i32 = 8;
const MIN_SPLIT: i32 = 16;

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
}

impl Allocator {
    fn new() -> Self {
        Allocator {
            blocks: Vec::new(),
            head: None,
        }
    }

    fn init(&mut self, size: i32) {
        self.blocks.clear();
        self.head = None;
        let usable = size - HEAD_SIZE;
        if usable > 0 {
            self.blocks.push(Block {
                addr: HEAD_SIZE,
                size: usable,
                used: false,
                prev: None,
                next: None,
            });
            self.head = Some(0);
        }
    }

    // First-fit: walk the list in address order, split the winning block if
    // the remainder is big enough to be worth tracking on its own.
    fn alloc(&mut self, req: i32) -> Option<i32> {
        let mut cur = self.head;
        while let Some(idx) = cur {
            let blk = &self.blocks[idx];
            cur = blk.next;
            if blk.used || blk.size < req {
                continue;
            }

            let addr = blk.addr;
            let remainder = blk.size - req;
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

            return Some(addr);
        }
        None
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

    fn print_blocks(&self) {
        let mut cur = self.head;
        while let Some(idx) = cur {
            let b = &self.blocks[idx];
            println!("{}:{}:{}", b.addr, b.size, if b.used { "used" } else { "free" });
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
                alloc.init(size);
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
            "BLOCKS" => alloc.print_blocks(),
            _ => println!("POOP"),
        }
    }
}
