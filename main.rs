use std::io::{self, BufRead};

const HEADER: i32 = 8;
const FOOTER: i32 = 8;
const OVERHEAD: i32 = HEADER + FOOTER;
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
        let usable = size - OVERHEAD;
        if usable > 0 {
            self.blocks.push(Block {
                addr: HEADER,
                size: usable,
                used: false,
                prev: None,
                next: None,
            });
            self.head = Some(0);
        }
    }

    fn find_idx(&self, addr: i32) -> Option<usize> {
        let mut cur = self.head;
        while let Some(idx) = cur {
            if self.blocks[idx].addr == addr {
                return Some(idx);
            }
            cur = self.blocks[idx].next;
        }
        None
    }

    // Every carved-off block now costs a real header+footer (boundary tags),
    // so only split when the leftover still covers that overhead plus a
    // worthwhile payload — otherwise the free remainder becomes internal
    // fragmentation inside the used block, same spirit as the plain MIN_SPLIT
    // rule from the splitting lesson.
    fn alloc(&mut self, req: i32) -> Option<i32> {
        let mut cur = self.head;
        while let Some(idx) = cur {
            let blk = &self.blocks[idx];
            cur = blk.next;
            if blk.used || blk.size < req {
                continue;
            }

            let addr = blk.addr;
            let leftover = blk.size - req;
            self.blocks[idx].used = true;

            if leftover - OVERHEAD >= MIN_SPLIT {
                let old_next = self.blocks[idx].next;
                self.blocks[idx].size = req;

                let new_idx = self.blocks.len();
                self.blocks.push(Block {
                    addr: addr + req + OVERHEAD,
                    size: leftover - OVERHEAD,
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
        match self.find_idx(addr) {
            Some(idx) if self.blocks[idx].used => {
                self.blocks[idx].used = false;
                self.coalesce(idx);
                true
            }
            _ => false,
        }
    }

    // Boundary tags are what make this O(1) in a real allocator: the footer
    // just behind a block's header names its neighbor directly instead of
    // requiring a scan. Our arena's prev/next links serve that same role
    // here, since the list is always kept in address order.
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

    fn describe(&self, idx: usize) -> String {
        let b = &self.blocks[idx];
        format!("{}:{}:{}", b.addr, b.size, if b.used { "used" } else { "free" })
    }

    fn print_blocks(&self) {
        let mut cur = self.head;
        while let Some(idx) = cur {
            println!("{}", self.describe(idx));
            cur = self.blocks[idx].next;
        }
    }

    fn count(&self) -> usize {
        let mut cur = self.head;
        let mut n = 0;
        while let Some(idx) = cur {
            n += 1;
            cur = self.blocks[idx].next;
        }
        n
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
            "COUNT" => println!("{}", alloc.count()),
            "PREV" => {
                let addr = spl[1].parse::<i32>().unwrap();
                match alloc.find_idx(addr) {
                    None => println!("BAD"),
                    Some(idx) => match alloc.blocks[idx].prev {
                        None => println!("NONE"),
                        Some(p) => println!("{}", alloc.describe(p)),
                    },
                }
            }
            "NEXT" => {
                let addr = spl[1].parse::<i32>().unwrap();
                match alloc.find_idx(addr) {
                    None => println!("BAD"),
                    Some(idx) => match alloc.blocks[idx].next {
                        None => println!("NONE"),
                        Some(n) => println!("{}", alloc.describe(n)),
                    },
                }
            }
            _ => println!("POOP"),
        }
    }
}
