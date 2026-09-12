use std::io::{self, BufRead};

// Keyword-based classifier: match a one-line workload description to the
// real-world allocator strategy best suited to it. Each list holds
// substrings (checked case-insensitively) that are strong signals for
// that strategy; the strategy with the most hits wins, ties broken by
// list order below, and no hits at all falls back to "default".
fn classify(desc: &str) -> &'static str {
    let d = desc.to_lowercase();

    let slab_kw = [
        "dentry",
        "inode",
        "task_struct",
        "kernel object",
        "object cache",
        "fixed-size object",
        "fixed size object",
        "same-size",
        "same size",
        "object pool",
        "packet buffer",
        "network packet",
        "slab",
    ];
    let tcmalloc_kw = [
        "thread",
        "threads",
        "threaded",
        "concurrent",
        "concurrency",
        "multi-threaded",
        "multithreaded",
        "parallel",
        "per-cpu",
        "lock contention",
        "lock-free",
        "tcmalloc",
    ];
    let buddy_kw = [
        "physical page",
        "physical memory page",
        "power-of-two",
        "power of two",
        "buddy system",
        "buddy allocator",
        "page allocator",
        "virtual memory page",
    ];
    let arena_kw = [
        "short-lived",
        "short lived",
        "per-frame",
        "per frame",
        "frame",
        "parser",
        "parsing",
        "transient",
        "scratch",
        "temporary",
        "bump allocat",
        "arena",
        "request-scoped",
    ];
    let default_kw = [
        "general-purpose",
        "general purpose",
        "desktop app",
        "desktop application",
        "variable size",
        "varied size",
        "mixed workload",
    ];

    let score = |kws: &[&str]| kws.iter().filter(|k| d.contains(*k)).count();

    // Iterator::max_by_key keeps the *last* of equally-scored candidates,
    // so this is listed lowest-priority-first: on a tie, slab beats
    // tcmalloc beats buddy beats arena beats default.
    let candidates: [(&str, usize); 5] = [
        ("default", score(&default_kw)),
        ("arena", score(&arena_kw)),
        ("buddy", score(&buddy_kw)),
        ("tcmalloc", score(&tcmalloc_kw)),
        ("slab", score(&slab_kw)),
    ];

    match candidates.iter().max_by_key(|&&(_, s)| s) {
        Some(&(name, s)) if s > 0 => name,
        _ => "default",
    }
}

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let l = line.unwrap();
        if l.is_empty() {
            continue;
        }
        println!("{}", classify(&l));
    }
}
