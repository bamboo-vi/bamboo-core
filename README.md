# Bamboo Core (Rust)

[![Crates.io](https://img.shields.io/crates/v/bamboo-core.svg)](https://crates.io/crates/bamboo-core)
[![Documentation](https://docs.rs/bamboo-core/badge.svg)](https://docs.rs/bamboo-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Vietnamese input method engine (IME) core written in Rust, ported from [bamboo-core](https://github.com/BambooEngine/bamboo-core) (Go).

## Features

- **Telex, VNI, VIQR** input methods with custom input method support
- **Extended phonetic onsets**: accepts `f`, `j`, and `w` as initial consonants alongside `z`
- **Hybrid engine**: Rule-based transformations + Lazy JIT DFA caching
- **SWAR / Vectorized matching**: Branchless 8-byte word lookups for transition routing
- **Parallel batch processing**: Rayon-powered work-stealing for bulk text processing (`parallel` feature)
- **Zero heap allocation** in core processing path (stack-allocated buffers)
- **O(1) backspace** via compact 8-byte snapshot stack
- **O(N) single-pass** spelling validation
- **FFI** for C/C++ integration and **WASM** bindings
- **Legacy charset conversion** to and from Unicode for 16 Vietnamese encodings

## Installation

```toml
[dependencies]
bamboo-core = "0.5.5"
```

### Legacy Charset Conversion (0.5.1)

`encode_charset` converts Unicode to a legacy Vietnamese encoding, while
`decode_charset` converts supported legacy text back to Unicode. Decoding uses the longest
matching sequence and leaves ambiguous codes unchanged rather than guessing.

```rust
use bamboo_core::{Charset, decode_charset, encode_charset};

let viqr = encode_charset(Charset::Viqr, "mưa.");
assert_eq!(viqr, "mu+a\\.");
assert_eq!(decode_charset(Charset::Viqr, &viqr), "mưa.");

assert_eq!(
    decode_charset(Charset::VniWindows, "tieáng Vieät"),
    "tiếng Việt"
);
```

VIQR punctuation that would otherwise extend the preceding letter is escaped with `\\`, so
encoding and decoding preserve ordinary prose such as sentence-ending full stops and questions.

### Migration Notes from 0.5.0

Version 0.5.0 introduces named APIs for operations whose boolean arguments were easy to
misread. The old APIs remain source-compatible, emit deprecation warnings, and are planned
for removal in 1.0.0.

| Deprecated API | Replacement |
|---|---|
| `remove_last_char(true)` | `undo_last_key()` or `undo_last_key_with(ToneRefresh::Recompute)` |
| `remove_last_char(false)` | `undo_last_key_with(ToneRefresh::PreserveSnapshot)` |
| `restore_last_word(true)` | `restore_last_word_as(LastWordForm::Vietnamese)` |
| `restore_last_word(false)` | `restore_last_word_as(LastWordForm::RawKeystrokes)` |
| `is_valid(true)` | `is_valid_word()` |
| `is_valid(false)` | `is_valid_prefix()` |
| `std_tone_style(true)` | `tone_style(ToneStyle::Modern)` |
| `std_tone_style(false)` | `tone_style(ToneStyle::Traditional)` |
| `process_key_delta(...)` | `process_key_edit(...)`, returning named `EditDelta` fields |
| `process_key_delta_into(...)` | `process_key_edit_into(...)` |

Boolean configuration switches such as `auto_correct(bool)` and `free_tone_marking(bool)`
remain unchanged because they express ordinary enabled/disabled state.

## C and C++ integration

The crate ships `include/bamboo_core.h` and builds both `libbamboo_core.a` and
`libbamboo_core.so`. Use a `BambooEngine` per input context; it owns only composition state.
The host application remains responsible for preedit presentation, committing text, key-event
routing, dictionaries, macros, and platform integration.

```c
#include <bamboo_core.h>

BambooEngine *engine = bamboo_engine_new(BAMBOO_METHOD_TELEX);
char *preedit = bamboo_engine_process_key_mode(
    engine, 'a', BAMBOO_MODE_VIETNAMESE);
/* Use preedit, then release it with the allocator from this library. */
bamboo_free_string(preedit);
bamboo_engine_free(engine);
```

For a normal keystroke loop, prefer `bamboo_engine_process_key_buf`: it returns the insert text
and deletion counts in caller-owned storage without allocating. Provide at least
`BAMBOO_MAX_EDIT_BYTES` bytes; a smaller buffer returns `1` without consuming the key, so the
caller can retry safely with a larger buffer. Committed text is retained for `FULL_TEXT` output by
default; call `bamboo_engine_set_full_text_tracking(engine, 0)` to release that history when it is
not needed. The process-wide `bamboo_*`
entrypoints and `bamboo_engine_process` are deprecated; compilers that support deprecation
attributes warn when the C header is used.

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

> ⚠️ **Note:** `Engine::process` is a convenience wrapper intended for **testing and batch validation only** (not for real-time production IME integration). For production applications, use `process_key` or `process_key_edit`.

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

For efficient text editor integration, use `process_key_edit` to get a named suffix edit:

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());

let edit = engine.process_key_edit('a', Mode::Vietnamese);
assert_eq!(edit.delete_chars, 0);
assert_eq!(edit.insert, "a");

// previous = "a", new = "á"
let edit = engine.process_key_edit('s', Mode::Vietnamese);
assert_eq!(edit.delete_chars, 1); // delete 1 char ("a")
assert_eq!(edit.insert, "á");    // insert "á"
// result: "" + "á" = "á"
```

Contract:
```text
previous = [common_prefix] + [delete_chars chars to delete]
new      = [common_prefix] + [insert]
```
Frontend does not need to compute LCP — the engine does it.

## Backspace

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());
engine.process_str("chuyeenr", Mode::Vietnamese);
assert_eq!(engine.output(), "chuyển");

engine.undo_last_key();
assert_eq!(engine.output(), "chuyên");
```

Two backspace modes are available:

- `undo_last_key()` — undo the **last keystroke** (O(1) via snapshot stack).
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

Bamboo Core is architected for zero heap allocations in the interactive typing loop, sub-microsecond keystroke latency, and high CPU cache efficiency via L1 cache line packing and SWAR vector matching. In `v0.5.4`, the `Engine` footprint is 1096 B (hot-first field layout, 32-transformation window, no persisted scratch stacks, lazy DFA arenas, 8-byte snapshots), with an L1-resident working set well under 4 KB per keystroke; DFA states are 64 B single-cache-line (popcount-rank transitions, no key array).

| Benchmark Scenario | Bamboo Core `0.5.4` | Uvie `2.8.1` | Bamboo/Uvie |
|---|---:|---:|---:|
| Feed single word (`tieengs`) | 199.8 ns | 776.4 ns | 0.26× |
| Feed no warmup (DFA miss) | 6.044 µs | 1.013 µs | 5.97× |
| Feed + delta (`tieengs`) | 263.8 ns | 807.1 ns | 0.33× |
| Backspace (`tieengs`) | 218.6 ns | 997.8 ns | 0.22× |
| Backspace x3 | 245.5 ns | 1.389 µs | 0.18× |
| Mixed typing (32 chars) | 977.3 ns | 3.413 µs | 0.29× |
| Many words (13 words + spaces) | 2.445 µs | 10.305 µs | 0.24× |
| English passthrough (code) | 144.3 ns | 1.311 µs | 0.11× |
| Long identifier (39 chars) | 334.0 ns | 11.279 µs | 0.03× |
| Backspace spam (7x) | 287.1 ns | 1.584 µs | 0.18× |
| Commit via space | 211.2 ns | 502.6 ns | 0.42× |
| Worst-case syllable (`nghieeng`) | 121.3 ns | 664.3 ns | 0.18× |
| Random typing (real workload) | 4.432 µs | 8.105 µs | 0.55× |

*Benchmarks from `benches/engine_bench.rs`, medians of 3 runs of 500,000 iterations each, on x86_64 Linux with an AMD Ryzen 7 H 255 and Rust 1.98.0, using the Fat LTO bench profile. Ratios are Bamboo Core time divided by Uvie time; lower than 1 means Bamboo Core is faster. Each `op` is one full workload iteration, not one keystroke.*

> **Note for developers:** bench binaries run a fast smoke under `cargo test --all-targets`; the full suites (including multi-minute EXTREME memory profiles) run only under `cargo bench`. Per-keystroke polling loops should prefer `get_processed_str_cow` over `get_processed_str` to borrow cached output instead of allocating.

## Architecture

The codebase follows a modular domain-driven architecture designed for zero allocations and extreme CPU cache locality:

- **`engine` (`src/engine/`)**: Core state machine managing active syllable compositions (`TransformationStack`), Counting Sort pre-partitioned rule index tables (`EngineRules`), $O(1)$ keystroke rollback snapshots (8-byte `Snapshot`), and word restoration.
- **`input_method` (`src/input_method/`)**: Input method definitions (Telex, VNI, VIQR, Microsoft layout) with zero-copy rule sharing across instances via `Arc<EngineRules>` and `Arc<InputMethod>`.
- **`orthography` (`src/orthography/`)**: Vietnamese orthography domain covering character phonetics, dense diacritic and tone tables, $O(1)$ bitmask syllable validation (`spelling`), and CVC syllable boundary extraction.
- **`dfa` (`src/dfa/`)**: JIT cache with flat arena storage (lazy arenas, 8192-state freeze cap), 128-bit bitset fast rejection, branchless popcount-rank transition lookup, 64 B single-cache-line states, and canvas flattener.
- **`encoder` (`src/encoder/`)**: Legacy encoding and decoding for 16 Vietnamese character sets (TCVN3, VNI-Windows, VIQR, VISCII, VPS, etc.); the typed `Cow` APIs borrow unchanged text.
- **`ffi` (`src/ffi.rs`) & `wasm` (`src/wasm.rs`)**: Safe C-ABI bindings with `#![deny(unsafe_op_in_unsafe_fn)]` and WebAssembly wrappers.

## License

MIT License. See [LICENSE](LICENSE) for details.
