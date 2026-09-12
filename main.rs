use std::io::{self, BufRead};

const CLASSES: [i32; 7] = [16, 32, 64, 128, 256, 512, 1024];

struct SizeClasses {
    alloc: [i32; 7],
    free: [i32; 7],
}

impl SizeClasses {
    fn new() -> Self {
        SizeClasses {
            alloc: [0; 7],
            free: [0; 7],
        }
    }

    fn class_for(size: i32) -> Option<usize> {
        CLASSES.iter().position(|&c| c >= size)
    }

    fn index_of(class: i32) -> Option<usize> {
        CLASSES.iter().position(|&c| c == class)
    }

    // Reuse a freed block from the class's own free list before minting a
    // new one, so a free/alloc round-trip never fabricates phantom objects.
    fn alloc(&mut self, size: i32) -> Option<i32> {
        let idx = Self::class_for(size)?;
        if self.free[idx] > 0 {
            self.free[idx] -= 1;
        }
        self.alloc[idx] += 1;
        Some(CLASSES[idx])
    }

    fn free(&mut self, class: i32) -> bool {
        match Self::index_of(class) {
            Some(idx) if self.alloc[idx] > 0 => {
                self.alloc[idx] -= 1;
                self.free[idx] += 1;
                true
            }
            _ => false,
        }
    }

    fn print_stats(&self) {
        for i in 0..CLASSES.len() {
            println!("{}:alloc={}:free={}", CLASSES[i], self.alloc[i], self.free[i]);
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let mut sc = SizeClasses::new();

    for line in stdin.lock().lines() {
        let l = line.unwrap();
        if l.is_empty() {
            continue;
        }

        let spl: Vec<&str> = l.split(' ').collect();

        match spl[0] {
            "ALLOC" => {
                let size = spl[1].parse::<i32>().unwrap();
                match sc.alloc(size) {
                    Some(class) => println!("class={}", class),
                    None => println!("TOO_LARGE"),
                }
            }
            "FREE" => {
                let class = spl[1].parse::<i32>().unwrap();
                if sc.free(class) {
                    println!("OK");
                } else {
                    println!("BAD");
                }
            }
            "STATS" => sc.print_stats(),
            _ => println!("POOP"),
        }
    }
}
