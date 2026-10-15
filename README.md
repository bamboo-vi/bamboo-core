# Bamboo Core (Rust)

[![Crates.io](https://img.shields.io/crates/v/bamboo-core.svg)](https://crates.io/crates/bamboo-core)
[![Documentation](https://docs.rs/bamboo-core/badge.svg)](https://docs.rs/bamboo-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Vietnamese input method engine (IME) core written in Rust, ported from [bamboo-core](https://github.com/BambooEngine/bamboo-core) (Go).

## Features

- **Telex, VNI, VIQR** input methods with custom input method support
- **Hybrid engine**: Rule-based transformations + Lazy JIT DFA caching
- **SWAR / Vectorized matching**: Branchless 8-byte word lookups for transition routing
- **Parallel batch processing**: Rayon-powered work-stealing for bulk text processing (`parallel` feature)
- **Zero heap allocation** in core processing path (stack-allocated buffers)
- **O(1) backspace** via compact 8-byte snapshot stack
- **O(N) single-pass** spelling validation
- **FFI** for C/C++ integration and **WASM** bindings

## Installation

```toml
[dependencies]
bamboo-core = "0.4.1"
```

## Quick Start — IME Integration

Feed keystrokes one at a time with `process_key`:

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());

engine.process_key('t', Mode::Vietnamese);
engine.process_key('i', Mode::Vietnamese);
engine.process_key('e', Mode::Vietnamese);
engine.process_key('e', Mode::Vietnamese);
engine.process_key('n', Mode::Vietnamese);
engine.process_key('g', Mode::Vietnamese);
engine.process_key('s', Mode::Vietnamese);
assert_eq!(engine.output(), "tiếng");
```

### Batch & Testing Convenience

> ⚠️ **Note:** `Engine::process` is a convenience wrapper intended for **testing and batch validation only** (not for real-time production IME integration). For production applications, use `process_key` or `process_key_delta`.

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());

// Testing convenience wrapper
let word = engine.process("tieengs", Mode::Vietnamese);
assert_eq!(word, "tiếng");

engine.reset();
let word2 = engine.process("vieetj", Mode::Vietnamese);
assert_eq!(word2, "việt");
```

## Tone Key Semantics

In Telex, a new tone key always replaces the previous tone on the same vowel —
the last tone key wins, even when tone keys are interleaved with letters or
other tone keys:

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());
assert_eq!(engine.process("looixfsx", Mode::Vietnamese), "lỗi");
```

Typing the same tone key twice in a row is an intentional undo: the tone is
removed and the key is typed as a literal letter.

```rust
let mut engine = Engine::new(InputMethod::telex());
assert_eq!(engine.process("ass", Mode::Vietnamese), "as");
```

## Delta Updates for Text Editors

For efficient text editor integration, use `process_key_delta` to get a **3-way diff**:

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());

let (bs, _, ins) = engine.process_key_delta('a', Mode::Vietnamese);
assert_eq!(bs, 0);
assert_eq!(ins, "a");

// previous = "a", new = "á"
let (bs, _, ins) = engine.process_key_delta('s', Mode::Vietnamese);
assert_eq!(bs, 1);     // delete 1 char ("a")
assert_eq!(ins, "á");  // insert "á"
// result: "" + "á" = "á"
```

Contract:
```text
previous = [common_prefix] + [backspace_count chars to delete]\nnew      = [common_prefix] + [inserted_suffix]
```
Frontend does not need to compute LCP — the engine does it.

## Backspace

```rust
use bamboo_core::{Engine, Mode, InputMethod, RestoreMark};

let mut engine = Engine::new(InputMethod::telex());
engine.process_str("chuyeenr", Mode::Vietnamese);
assert_eq!(engine.output(), "chuyển");

engine.remove_last_char(RestoreMark::Yes); // or pass true
assert_eq!(engine.output(), "chuyên");
```

Two backspace modes are available:

- `remove_last_char(RestoreMark::Yes)` (or `true`) — undo the **last keystroke** (O(1) via snapshot stack).
  `tiếng` + DEL -> `tiêng`.
- `remove_last_output_char()` — delete the **entire character before the caret**,
  keeping mark/tone transformations on earlier characters. `tiếng` + DEL -> `tiến`:

```rust
let mut engine = Engine::new(InputMethod::telex());
engine.process_str("tieesng", Mode::Vietnamese);
assert_eq!(engine.output(), "tiếng");

engine.remove_last_output_char();
assert_eq!(engine.output(), "tiến");
```

## Output Customization

```rust
use bamboo_core::{Engine, Mode, InputMethod, OutputOptions};

let mut engine = Engine::new(InputMethod::telex());
engine.process_str("Trangws", Mode::Vietnamese);

// Toneless
assert_eq!(engine.get_processed_str(OutputOptions::TONE_LESS), "Trăng");

// Full text (committed + active)
assert_eq!(engine.get_processed_str(OutputOptions::FULL_TEXT), "Trăng");
```

## Performance & Benchmarks

Bamboo Core is architected for zero heap allocations in the interactive typing loop, sub-microsecond keystroke latency, and high CPU cache efficiency via L1 cache line packing and SWAR vector matching. On current `main` the `Engine` footprint is 1096 B (hot-first field layout, 32-transformation window, no persisted scratch stacks, lazy DFA arenas, 8-byte snapshots), with an L1-resident working set well under 4 KB per keystroke; DFA states are 64 B single-cache-line (popcount-rank transitions, no key array).

| Benchmark Scenario | `v0.4.0` | `v0.4.1` | Delta |
|---|---|---|---|
| Feed single word (`tieengs`) | 313.2 ns/op | **278.6 ns/op** | **-11.0%** |
| Feed no warmup (DFA miss) | 6.235 µs/op | **6.001 µs/op** | **-3.7%** |
| Feed + delta (per key) | 400.3 ns/op | **343.1 ns/op** | **-14.3%** |
| Backspace (`tieengs`) | 350.5 ns/op | **300.9 ns/op** | **-14.2%** |
| Backspace x3 | 389.6 ns/op | **345.4 ns/op** | **-11.3%** |
| Mixed typing (29 chars) | 1.467 µs/op | **1.248 µs/op** | **-14.9%** |
| Many words (13 words + spaces) | 9.504 µs/op | **9.472 µs/op** | **-0.3%** |
| English passthrough (code) | 320.2 ns/op | **186.3 ns/op** | **-41.8%** |
| Long identifier (39 chars) | 791.4 ns/op | **349.2 ns/op** | **-55.9%** |
| Backspace spam (7x) | 445.5 ns/op | **392.9 ns/op** | **-11.8%** |
| Commit via space | 308.5 ns/op | **284.8 ns/op** | **-7.7%** |
| Worst-case syllable (`nghieengs`) | 260.8 ns/op | **233.4 ns/op** | **-10.5%** |
| Random typing (real workload) | 5.578 µs/op | **5.162 µs/op** | **-7.5%** |

*Benchmarks from `benches/engine_bench.rs`, medians of 3 runs of 500k iterations, x86_64 Linux, Rust 1.88+ release profile with Fat LTO.*

> **Note for developers:** bench binaries run a fast smoke under `cargo test --all-targets`; the full suites (including multi-minute EXTREME memory profiles) run only under `cargo bench`. Per-keystroke polling loops should prefer `get_processed_str_cow` over `get_processed_str` to borrow cached output instead of allocating.

## Architecture

The codebase follows a modular domain-driven architecture designed for zero allocations and extreme CPU cache locality:

- **`engine` (`src/engine/`)**: Core state machine managing active syllable compositions (`TransformationStack`), Counting Sort pre-partitioned rule index tables (`EngineRules`), $O(1)$ keystroke rollback snapshots (8-byte `Snapshot`), and word restoration.
- **`input_method` (`src/input_method/`)**: Input method definitions (Telex, VNI, VIQR, Microsoft layout) with zero-copy rule sharing across instances via `Arc<EngineRules>` and `Arc<InputMethod>`.
- **`orthography` (`src/orthography/`)**: Vietnamese orthography domain covering character phonetics, dense diacritic and tone tables, $O(1)$ bitmask syllable validation (`spelling`), and CVC syllable boundary extraction.
- **`dfa` (`src/dfa/`)**: JIT cache with flat arena storage (lazy arenas, 8192-state freeze cap), 128-bit bitset fast rejection, branchless popcount-rank transition lookup, 64 B single-cache-line states, and canvas flattener.
- **`encoder` (`src/encoder/`)**: Zero-allocation legacy encoding conversion supporting 16 Vietnamese character sets (TCVN3, VNI-Windows, VIQR, VISCII, VPS, etc.).
- **`ffi` (`src/ffi.rs`) & `wasm` (`src/wasm.rs`)**: Safe C-ABI bindings with `#![deny(unsafe_op_in_unsafe_fn)]` and WebAssembly wrappers.

## License

MIT License. See [LICENSE](LICENSE) for details.
