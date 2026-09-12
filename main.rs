use std::io::{self, BufRead};

// All allocations are rounded up to this alignment; the gap between the
// requested size and the rounded size is what REPORT counts as internal
// fragmentation.
const ALIGN: i64 = 16;

fn round_up(size: i64) -> i64 {
    let size = size.max(1);
    ((size + ALIGN - 1) / ALIGN) * ALIGN
}

struct Block {
    addr: i64,
    size: i64,
    used: bool,
    // Only meaningful while `used` — the size actually requested, before
    // rounding, so REPORT can recover per-block internal fragmentation.
    requested: i64,
}

struct Heap {
    blocks: Vec<Block>,
}

impl Heap {
    fn new(total: i64) -> Self {
        Heap {
            blocks: vec![Block {
                addr: 0,
                size: total,
                used: false,
                requested: 0,
            }],
        }
    }

    // First-fit: take the first free block (in address order) big enough,
    // splitting off the leftover as a new free block when it doesn't fit
    // exactly.
    fn alloc(&mut self, requested: i64) -> Option<i64> {
        let rounded = round_up(requested);
        let idx = self
            .blocks
            .iter()
            .position(|b| !b.used && b.size >= rounded)?;

        let addr = self.blocks[idx].addr;
        let leftover = self.blocks[idx].size - rounded;

        self.blocks[idx].size = rounded;
        self.blocks[idx].used = true;
        self.blocks[idx].requested = requested;

        if leftover > 0 {
            self.blocks.insert(
                idx + 1,
                Block {
                    addr: addr + rounded,
                    size: leftover,
                    used: false,
                    requested: 0,
                },
            );
        }

        Some(addr)
    }

    // Free the block at `addr`, then coalesce with a free neighbor on
    // either side so adjacent free space doesn't stay fragmented.
    fn free(&mut self, addr: i64) -> bool {
        let idx = match self.blocks.iter().position(|b| b.addr == addr && b.used) {
            Some(i) => i,
            None => return false,
        };

        self.blocks[idx].used = false;
        self.blocks[idx].requested = 0;

        if idx + 1 < self.blocks.len() && !self.blocks[idx + 1].used {
            let next_size = self.blocks[idx + 1].size;
            self.blocks[idx].size += next_size;
            self.blocks.remove(idx + 1);
        }

        if idx > 0 && !self.blocks[idx - 1].used {
            self.blocks[idx - 1].size += self.blocks[idx].size;
            self.blocks.remove(idx);
        }

        true
    }

    fn report(&self) -> String {
        let mut used = 0i64;
        let mut free = 0i64;
        let mut free_blocks = 0i64;
        let mut largest_free = 0i64;
        let mut internal_frag = 0i64;

        for b in &self.blocks {
            if b.used {
                used += b.size;
                internal_frag += b.size - b.requested;
            } else {
                free += b.size;
                free_blocks += 1;
                largest_free = largest_free.max(b.size);
            }
        }

        let external_frag = if free > 0 {
            1.0 - (largest_free as f64 / free as f64)
        } else {
            0.0
        };

        format!(
            "used={} free={} free_blocks={} largest_free={} internal_frag={} external_frag={:.4}",
            used, free, free_blocks, largest_free, internal_frag, external_frag
        )
    }
}

fn main() {
    let stdin = io::stdin();
    let mut heap: Option<Heap> = None;

    for line in stdin.lock().lines() {
        let l = line.unwrap();
        if l.is_empty() {
            continue;
        }

        let spl: Vec<&str> = l.split(' ').collect();

        match spl[0] {
            "INIT" => {
                let total = spl[1].parse::<i64>().unwrap();
                heap = Some(Heap::new(total));
                println!("OK");
            }
            "ALLOC" => {
                let size = spl[1].parse::<i64>().unwrap();
                match heap.as_mut().and_then(|h| h.alloc(size)) {
                    Some(addr) => println!("{}", addr),
                    None => println!("OOM"),
                }
            }
            "FREE" => {
                let addr = spl[1].parse::<i64>().unwrap();
                let ok = heap.as_mut().map_or(false, |h| h.free(addr));
                println!("{}", if ok { "OK" } else { "BAD" });
            }
            "REPORT" => {
                if let Some(h) = heap.as_ref() {
                    println!("{}", h.report());
                }
            }
            _ => println!("POOP"),
        }
    }
}
