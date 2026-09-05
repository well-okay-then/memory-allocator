use std::io::{self, BufRead};

// TODO (why-allocator): implement per the lesson description.

fn main() {
    let stdin = io::stdin();
    let mut size = 0;
    let mut cur = 0;
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
                let temp = cur;
                let t = spl[1].parse::<i32>().unwrap();
                if t + cur > size {
                    println!("OOM");
                } else {
                    cur += t;
                    println!("{}", temp);
                }
            }
            "USED" => {
                println!("{}", cur);
            }
            "RESET" => {
                cur = 0;
                println!("OK");
            }
            _ => {
                println!("POOP")
            }
        }
    }
}
