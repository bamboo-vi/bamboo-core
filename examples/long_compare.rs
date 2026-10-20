use bamboo_core::{Config, Engine, InputMethodPreset, Mode};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

struct CountingAllocator;
static ALLOCS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

fn rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|line| line.starts_with("VmRSS:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|n| n.parse().ok())
        })
        .unwrap_or(0)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let kind = args.get(1).map(String::as_str).unwrap_or("reset");
    let n: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(10_000);
    let words = ["tieengs", "vieetj", "nguwowif", "chuyeenr"];
    let mut e = if std::env::var_os("BAMBOO_AUTO_CORRECT_OFF").is_some() {
        Engine::from_preset_with_config(
            InputMethodPreset::Telex,
            Config::builder().auto_correct(false).build(),
        )
    } else {
        Engine::from_preset(InputMethodPreset::Telex)
    };
    let base_allocs = ALLOCS.load(Ordering::Relaxed);
    let base_bytes = BYTES.load(Ordering::Relaxed);
    let mut ops = 0u64;
    let mut checksum = 0u64;
    let start = Instant::now();
    let mut last = start;
    for i in 1..=n {
        if kind == "reset" {
            e.reset();
        }
        for c in words[(i - 1) % words.len()].chars() {
            e.process_key(c, Mode::Vietnamese);
            checksum = checksum.wrapping_add(e.output_str().len() as u64);
            ops += 1;
        }
        if kind == "session" {
            e.process_key(' ', Mode::Vietnamese);
            checksum = checksum.wrapping_add(e.output_str().len() as u64);
            ops += 1;
        }
        if i == 1_000 || i == 5_000 || i == 10_000 || i == 50_000 || i == n {
            let now = Instant::now();
            let ms = now.duration_since(last).as_secs_f64() * 1000.0;
            let total_ms = now.duration_since(start).as_secs_f64() * 1000.0;
            let allocations = ALLOCS.load(Ordering::Relaxed) - base_allocs;
            let bytes = BYTES.load(Ordering::Relaxed) - base_bytes;
            println!(
                "SAMPLE\t{kind}\t{i}\t{ops}\t{ms:.3}\t{total_ms:.3}\t{}\t{allocations}\t{bytes}\t{}\t{}\t{checksum}",
                rss_kib(),
                e.dfa_memory_allocated(),
                e.committed_text_len(),
            );
            last = now;
        }
    }
}
