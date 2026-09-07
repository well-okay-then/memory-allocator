use std::{
    collections::BTreeMap,
    io::{self, BufRead},
};

// TODO (why-allocator): implement per the lesson description.

fn main() {
    let stdin = io::stdin();
    let mut size = 0;
    let mut cur = 0;
    let head_size = 4;
    let mut addr_to_size = BTreeMap::new();
    for line in stdin.lock().lines() {
        let l = line.unwrap();
        if l.is_empty() {
            continue;
        }

        let spl: Vec<&str> = l.split(" ").collect();

        match spl[0] {
            "INIT" => {
                size = spl[1].parse::<i32>().unwrap();
                println!("OK");
            }
            "ALLOC" => {
                let t = spl[1].parse::<i32>().unwrap();
                if t + cur + head_size > size {
                    println!("OOM");
                } else {
                    cur += head_size;
                    let addr = cur;
                    println!("{}", cur);
                    cur += t;
                    addr_to_size.insert(
                        addr,
                        Mem {
                            size: t,
                            free: false,
                        },
                    );
                }
            }
            "FREE" => {
                let addr = spl[1].parse::<i32>().unwrap();
                if addr_to_size.contains_key(&addr) {
                    let s = addr_to_size.get(&addr).unwrap().size;
                    cur -= s;
                    cur -= head_size;
                    addr_to_size.insert(
                        addr,
                        Mem {
                            size: s,
                            free: true,
                        },
                    );
                    println!("OK")
                } else {
                    println!("BAD")
                }
            }
            "BLOCKS" => {
                for ele in addr_to_size.iter() {
                    let t = if ele.1.free { "free" } else { "used" };
                    println!("{}:{}:{}", ele.0, ele.1.size, t);
                }
            }
            _ => {
                println!("POOP")
            }
        }
    }
}

struct Mem {
    size: i32,
    free: bool,
}
