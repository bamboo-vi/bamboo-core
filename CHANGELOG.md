# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.5.2] - 2026-10-23

### Build

- Broadened the `phf` compatibility range to `>=0.11, <0.14`. bamboo-core uses APIs available
  throughout that range, allowing Debian unstable's `phf 0.11.3` without downstream patches.

### Documentation

- Added a crate-level rustdoc migration guide for 0.4.x callers. It lists every deprecated API,
  its 0.5 replacement, and the 1.0.0 removal horizon. The API selection guide now recommends the
  typed `encode_charset` and `decode_charset` APIs over `advanced::encode` for new code.

## [0.5.1] - 2026-10-23

### Added

- Added `decode_charset`, the typed inverse of `encode_charset`, and re-exported it from the
  crate root and `advanced` module. It decodes all supported legacy charset tables using
  longest-match parsing, preserves text with no encodings as borrowed `Cow`, and leaves
  irreducibly ambiguous codes unchanged.
- Added legacy-character-set coverage and Criterion workloads for VIQR and VNI decoding,
  including known external VISCII control-code capitals and VIQR backslash escapes.

### Fixed

- Corrected Windows-1258 output for `Ứ` and `Đ`.
- VIQR encoding now escapes a backslash and punctuation that would otherwise join the previous
  encoded letter. `encode_charset(Charset::Viqr, "mưa.")` now produces `mu+a\\.` and decodes
  back to the original text.

### Performance

- Lowercase DFA hits now copy their cached UTF-8 output directly instead of decoding it and
  walking the transformation buffer to reproduce identical text.
- ASCII DFA transition rank uses two native `u64` words instead of variable `u128` shifts.
- The two-word transition rank selects its lower-word contribution with a 0/1 mask, removing
  a data-dependent lane-selection branch. Against a same-session branchy build, five warmed
  typing streams showed 1.8% and 3.2% improvements on two traces, no significant change on
  two, and a 1.9% regression on one; the aggregate effect is small and mixed.
- Engine-owned DFA output skips repeated UTF-8 validation; the public advanced DFA reader
  remains checked because callers can mutate its public arenas.
- Steady-state allocation tests now require exactly zero allocations for warmed DFA input,
  English passthrough, edit deltas, output polling, and undo.
- Criterion workloads now reset to an identical state per iteration and exclude engine
  construction from the cold rule/JIT measurement. On the Ryzen 7 H 255 validation host,
  corrected benchmarks measured 18–27% faster DFA streams, 12% faster edit deltas,
  23–24% faster undo, and about 9% faster mixed sentences; cold JIT, English passthrough,
  uppercase input, and output polling showed no statistically significant regression.
- Decode checks whether the first character can begin an encoded sequence once per input
  position instead of repeating the lookup for every candidate length. On the release
  validation host, ASCII passthrough improved from 226 ns to 105 ns, VIQR decoding from
  1.50 µs to 0.92 µs, and VNI decoding from 1.75 µs to 0.87 µs.
- VIQR encoding skips charset-table lookups for ASCII input. The escaped-VIQR workload improved
  from 1.58 µs to 0.80 µs while preserving the same output.

## [0.5.0] - 2026-10-21

### Migration

- Replaced ambiguous behavioral booleans with named methods and enums. Existing callers
  continue to compile, but should migrate before 1.0.0:
  - `remove_last_char(true)` → `undo_last_key()` or
    `undo_last_key_with(ToneRefresh::Recompute)`.
  - `remove_last_char(false)` →
    `undo_last_key_with(ToneRefresh::PreserveSnapshot)`.
  - `restore_last_word(true/false)` →
    `restore_last_word_as(LastWordForm::Vietnamese/RawKeystrokes)`.
  - `is_valid(true/false)` → `is_valid_word()` / `is_valid_prefix()`.
  - `std_tone_style(true/false)` →
    `tone_style(ToneStyle::Modern/Traditional)`.
  - `process_key_delta` / `process_key_delta_into` →
    `process_key_edit` / `process_key_edit_into`.
- `auto_correct(bool)` and `free_tone_marking(bool)` remain boolean switches because their
  values have direct enabled/disabled meaning.

### Added

- Added named policy APIs that avoid ambiguous behavioral booleans: `undo_last_key`,
  `undo_last_key_with(ToneRefresh)`, `restore_last_word_as(LastWordForm)`,
  `is_valid_word`, `is_valid_prefix`, and `ConfigBuilder::tone_style(ToneStyle)`.
- Added `process_key_edit`, returning an `EditDelta` with named deletion and insertion fields,
  plus `process_key_edit_into` for caller-owned buffers.

### Deprecated

- Deprecated `remove_last_char`, `RestoreMark`, `restore_last_word(bool)`, `is_valid(bool)`,
  `process_key_delta`, `process_key_delta_into`, and `ConfigBuilder::std_tone_style(bool)`.
  Their `true` and `false` behavior is covered by compatibility tests; removal is planned
  for 1.0.0.

## [0.4.1] - 2026-10-15

### Fixed
- **Long identifiers no longer truncated or dropped:** the active-transformation window is widened from 16 to 32 and overflow now silently commits the head instead of cutting (`getUserNameByIdentifier` (23 chars) stays whole in the preedit; `TransformationStack::push` can no longer lose a key past the window). `active_upper_mask` widened `u16`→`u32`; `Snapshot` metadata stays 8 B (state IDs fit `u16` under the 8192-state DFA cap).
- **`uow` horn spread by onset class:** the `uơ` rime is only complete after the `c h k kh qu th` onset class (Go bamboo-core `cvMatrix`), so `duow`→`dươ`, `dduow`→`đươ`, `tuow`→`tươ`, `nguow`→`ngươ` while `khuow`→`khuơ` and bare `uowr`→`uở` stay as typed. The class-0 free-onset extension for `oă` (`boặm`, `toắt`, `nhoằng`) is preserved. Partial inputs stay valid via Go's row-union semantics.
- **Tone after both horns lands on `ơ`:** `tuowf`→`tườ`, `tuowfj`→`tượ` (was `từơ`/`tựơ`) — `find_tone_target` now prefers the vowel carrying `ơ/ê`, matching Go's `findToneTarget`.

### Performance
- **No scratch-stack zero-fill:** `TransformationStack` slots are `MaybeUninit` (construction of the two slow-path scratch stacks cost 2×256 B of zero-fill per keystroke at window 16, 2×512 B at 32 — the dominant regression source when widening).
- **Incremental preedit cache:** the raw-append (English/bypass) path grows `cached_output` O(1) per key instead of re-flattening the whole preedit (which was O(n²) over a long identifier).
- **Measured (`benches/engine_bench.rs`, median of 3×500k, vs 0.4.0):** all 13 scenarios at or faster than 0.4.0 — long identifier **-55.9%**, english passthrough **-41.8%**, mixed typing -14.9%, backspace -14.2%, feed single word -11.0%, worst-case syllable -10.5%, random typing -7.5%; no scenario regresses. Hot path stays 0-alloc (`alloc_regression` 9/9). `Engine` footprint 712 B → 1096 B (32-slot window), working set well under 4 KB.

### Internal
- **Phonology tables as data:** spelling data lives in `orthography/tables.rs` (single source of truth with how-to-extend notes) and `orthography/packing.rs` derives the `u16`-lane matchers at compile time. Tone-placement preferences (`TONE_PREFERRED`, `TONE_ON_SECOND`, `TONE_SECOND_ONSETS`) and `uơ`/`ưo` patterns (`UHO_PATTERNS`) are data tables instead of hard-coded matches.
- **Table well-formedness tests:** no duplicate tokens per row, mask dimensions match their row groups (`tables_wellformed`); overflow undo cannot cross the silent commit and the incremental preedit cache is locked to the full flatten (`tests/preedit_overflow.rs`).

## [0.4.0] - 2026-10-12

### Release notes
- **Version 0.4.0** closes out the long 0.3.x patch series. This is **not** a breaking release: the primary public API (`Engine`, `Config`, `InputMethod`, C-FFI) is fully backward compatible with 0.3.26. The only structural change is inside `advanced::State` internals (64 B layout, see below).

### Performance
- **Branchless popcount-rank DFA lookup:** `get_transition` is 1 bit-test + 1 `popcnt` rank with no data-dependent branches (was two 8-key SWAR chunks with 4 branches + tail gate; an intermediate 16-lane SWAR step was superseded mid-pass by the rank scheme). `trans_states` sorted by key, `trans_keys` dropped; `State` shrunk from 88 B to **64 B** (single cache line), `comp_hash` moved to a side table; `composition_matches_canonical` compares masked `u128` words. Also fixes a latent invariant bug: a 17th distinct key no longer sets its bitset bit without storing (`popcount(bitset) == trans_len` now holds).
- **Dense phonetics decode table:** `const` 2K-entry page table replaces PHF `VOWEL_INDEX`/`MARKS_MAPS` in `is_vowel`, `find_vowel_position`, `add_tone_to_char` (new `TONE_TABLE`), `add_mark_to_toneless_char`, `upper` and spelling `strip_mark`; table dimensions derive from `VOWELS.len()`.
- **One split per revision:** `extract_cvc_trans` runs once per slow-path keystroke (was 3–5×) and is threaded as `&Cvc` through `find_target`/`generate_undo`; `find_tone_target` reuses the stored split; `Cvc` carries absolute vowel indices (`vo_abs`) so vowel lookup no longer value-searches (mis-hit on identical twins, e.g. post-`gi`/`qu` split). Fixes latent `generate_transformations` uho fallback attaching to the wrong index on identical vowels.
- **Spelling validator packing:** tokens pre-packed to `u16` lanes with branchless per-lane matching (raw or stripped when incomplete) and early row exit; `VC_OK[256]` table replaces the `is_valid_vc` loop.
- **Flattener cleanup:** O(1) `len*4` reserve replaces the estimate pre-pass, loop-invariant options hoisted, effect chains built in application order via tail pointers (no collect + reverse), dead capacity guard removed; `get_applicable_rules` takes a pre-lowered key.
- **Measured (criterion, vs 2026-10-12 baseline):** keystroke streams -1% to -17% (`dduwowngf` -17%, `nguwowif` -9%), sentences -5% to -6%, backspace x3 -8.5%, commit -10%; no scenario regresses > 3%; hot path stays 0-alloc (`alloc_regression` green).

## [0.3.26] - 2026-10-09

### Features
- **Custom Input Methods:** `InputMethod::from_definition` builds an input method from runtime key → rule pairs in the preset format (e.g. `("q", "DauSac")`), like Go bamboo-core's `ParseInputMethod` on a user map.
- **`w` to `ư`:** `Config::w2u_mode` / `ConfigBuilder::w2u_mode` take a `W2uMode`: `Disabled` (default, unchanged behaviour), `NonStart` (`nhw` → `như`, a syllable-initial `w` stays `w`) or `Everywhere` (`w` → `ư`).
- **C configuration:** `bamboo_engine_new_with_flags(method, flags)` creates an engine from `Config::from_flags`; bits 3 (`NonStart`) and 4 (`Everywhere`) select the `w2u_mode`.
- **Rebuild From Text:** `Engine::rebuild_from_text` (FFI `bamboo_engine_rebuild_from_text`) loads existing text such as the word before the cursor, so the next tone, mark or backspace edits it as if it had been typed. Ported from Go bamboo-core `RebuildEngineFromText`; text up to the last word break is committed and the last word becomes the active composition.
- **Raw Full Text:** `FULL_TEXT | RAW` now returns the typed keys of committed words too, like Go's `GetProcessedString(EnglishMode | FullText)`. `rebuild_from_text` keeps `committed_raw` in step (rebuilt roots stand in for the unknowable original keystrokes). `FULL_TEXT` also honours `TONE_LESS`, `MARK_LESS` and `LOWER_CASE` for committed words and, like Go, ignores `PUNCTUATION_MODE`. Raw keys are appended once per `commit()`; plain `FULL_TEXT` stays zero-allocation when no word is active.
- **Brackets to `ơ`/`ư`:** `Config::bracket_mode` / `ConfigBuilder::bracket_mode` take a `BracketMode`: `Disabled` (default, unchanged behaviour), `NonStart` (`t[` → `tơ`, a word-initial bracket stays a bracket) or `Everywhere`. `[` `]` `{` `}` type `ơ` `ư` `Ơ` `Ư`; the same bracket twice gives the bracket back (`[[` → `[`). Telex 2, which maps brackets itself, is unchanged. Flag bits 5 (`NonStart`) and 6 (`Everywhere`) select the mode.
- **`Engine::can_process_key`:** tells a frontend whether a key takes part in composition, including brackets when `bracket_mode` enables them.

### Bug Fixes
- **Preset Rule Sharing:** `Engine::new`/`with_config` reuse a preset's shared rules only when the rules match, not just the name and rule count, so a custom input method named like a preset keeps its own keys.
- **`Engine::set_config`:** cached DFA transitions are dropped when the configuration changes, so new settings apply to words typed afterwards. The method is no longer `const` (breaking change for `const` contexts).
- **Undoing a mark keeps the tone:** typing a mark key again (`uwfw`, VNI `go366`) no longer drops the tone typed before it, so `uwfw` gives `ùw` instead of `uw`, as in the Go core.
- **Backspace back to a valid word:** after an undo switched a word to raw keys (`eete` → `ete`), deleting back to a valid word (`et`) lets the next key add marks again, so `eete`, backspace, `e` gives `êt` as in the Go core.
- **Backspace keeps the tone in place:** `remove_last_output_char` and `remove_last_char` move the tone to its standard position only when free tone marking is on and the remaining word is valid, as in the Go core. Deleting a key from an invalid word now brings back the text shown before that key (`craxyuk`, backspace gives `crãyu`, not `craỹu`).
- **Perf:** the tone-refresh validity check is skipped when the word has no tone to move, so backspacing a toneless word is faster than before.
- **Tone keys after an invalid word:** a key now edits only the part of the word after the last point where it stopped being a valid syllable, as in the Go core. A tone key after an invalid word is typed as a letter (`enlf` gives `enlf`, not `ènl`), and a tone stays on the syllable it was typed on (`mymfyk` gives `mỳmyk`, not `mymyk`).
- **Perf:** `last_syllable_start` resumes from a self-validating hint instead of re-checking every prefix, recovering most of the slow-path cost above.
- **Free onset–rime pairing:** any known onset now pairs with any rime (only the rime itself is constrained), so `krông`, `boặm`, `khuều` validate; the CV gate and its tables are removed.
- **Horn placement after `uo`:** the horn now goes on `o` when nothing follows (`khuow` gives `khuơ`, not `khuơ` with spread) and spreads to `u` only when a letter follows (`huouw` gives `hươu`), matching Go bamboo-core. Note: bare `uow` now gives `uơ` instead of `ươ`.

### Performance
- **`Engine` 1216 B → 688 B:** persisted scratch stacks removed (proven write-only), DFA arenas start unallocated (~20 KiB saved per fresh engine), hot fields first. Hit-path working set ≈ 360 B (L1-resident).
- **Zero-alloc polling:** `LOWER_CASE` reads borrow the DFA flat cache on warm words; batch `process_batch` reuses one engine per thread (26x per-item win over fresh-engine batching in microbench).
- **Slow path:** one shared `extract_cvc_trans` per validate-then-refresh keystroke; `last_syllable_start` resume hint; tone-presence guards before spelling/extraction work.
- **Hygiene:** dead code removed (`flatten_slice_into`, `pop`, unused rune/key helpers, `DfaCompiler` dead fields + lifetime); `#[must_use]` on pure APIs; `debug_assert` guards for narrowing casts and the incremental case mask; slow-path helpers split out of the 206-line `process_key_internal`.
- **Benches:** all six bench binaries run a fast smoke under `cargo test` (full suites are `cargo bench`-only), so `cargo test --all-targets` finishes in seconds instead of timing out on benchmark mains.

## [0.3.25] - 2026-09-25

### Performance & Memory
- **Compact Snapshot Stack:** `Snapshot` shrunk from 272 B to 8 B (`state_id` + `upper_mask` + `active_len` + flags), cutting the `Engine` footprint from ~5.3 KB to ~1.3 KB; the English bypass buffer is now lazily boxed.
- **Cache-Line DFA States:** `Dfa` state reduced from 160 B to 88 B so hot fields fit a single 64 B cache line, with `u16` state IDs.
- **Flattened Output Caching:** DFA states cache flattened lowercase output (`flat_arena`); a DFA hit becomes memcpy + case-apply instead of full effect-chain resolution.
- **Dense Phonetics Tables:** `MARK_TABLE [[char; 5]; 72]` replaces 4 PHF lookups per `add_mark_to_char` call with 1–2 array loads; `const fn strip_mark` removes PHF lookups from the spelling inner loop.
- **Hot-Path Cleanup:** Eliminated double `update_cached_output` on the slow path (~2x flatten cost), replaced heap `flatten_slice` with stack `uho_tail_match_composition`, hoisted `get_applicable_rules` and `extract_cvc_trans`, replaced the `[bool; 16]` case shuffle with a `u16` upper-mask bitmask, and removed the dead `uoh_tail_match`.

### Bug Fixes
- **Tone on Literal `oo` (VO_2):** Typing `thooongf` now yields `thoòng` — English bypass is gated on `is_valid` after mark/tone undo so words with literal `oo` still accept tones.
- **Tone Target Resolution:** `find_tone_target` uses absolute vowel indices and resolved mark chains so an undone `ô` no longer steals the tone target.
- **`extract_cvc_appending_indices`:** Fixed absolute-index bug (`results[fc[0]]` → `results[0]`).
- **DFA Hash Collisions:** `add_state` now backward-scans for previously inserted compositions instead of overwriting on collision.

### Testing & Infrastructure
- **Criterion Benchmarks:** New `benches/criterion_bench.rs` with `black_box`-sealed hot paths.
- **Allocation Regression Suite:** `tests/alloc_regression.rs` asserts zero heap allocations across 7 scenarios with a counting allocator.
- **Skey Parity Suite:** 16 tests ported from skey-engine (`tests/skey_parity.rs`), all passing.
- **Dependency Updates:** skey-engine 0.1.4 → 0.1.23, uvie 2.1.1 → 2.7.0.

## [0.3.24] - 2026-09-14

### Performance & Memory Architecture
- **Zero-Allocation Hot Path:** Stack-allocated active syllable composition (`[Transformation; MAX_ACTIVE_TRANS]`, where `MAX_ACTIVE_TRANS = 16`), achieving strictly 0 heap allocations during typing.
- **Cache-Optimized Transformation:** Aligned `Transformation` to exactly 16 bytes (128-bit boundary) with a static compile-time size assertion.
- **Zero-Allocation Output:** Cached preedit buffer enables `output_str(&self) -> &str` and `output(&self) -> Cow<'_, str>` without heap allocations.
- **Counting Sort Rule Partitioning:** Precomputed `EngineRules` with Counting Sort into direct-indexed ASCII slices (`[(u16, u16); 128]`) and partitioned non-ASCII rules, eliminating hot-path linear rule scans.
- **Zero-Copy Engine Preset Sharing:** Built-in presets (`Telex`, `VNI`, `VIQR`, etc.) now share rule tables via `Arc<EngineRules>` and `Arc<InputMethod>` stored in `LazyLock`, reducing `Engine` stack size from 6,176 B to 5,360 B (5.2 KB) and making `Engine::from_preset()` instantaneous with zero heap copying.
- **Sub-Microsecond Keystroke Latency:** Measured single-word latency down to ~140 ns ('vietj' -> 'việt') and full 27-character sentence streams at ~1.3 µs (8.7x faster than skey, up to 16x faster on paragraphs).

### Safety & FFI Invariants
- **Strict Safe FFI Boundaries:** Enforced `#![deny(unsafe_op_in_unsafe_fn)]` across `ffi.rs`.
- **Defensive Pointer Validation:** All FFI entrypoints validate pointers (`engine.as_mut()`) and catch panics gracefully with explicit `// SAFETY:` rationale comments.

### Domain-Driven Modularization
- **Clean Architecture:** Refactored monolithic flat source files into cohesive domain modules:
  - `src/engine/`: `mod.rs`, `rules.rs`, `snapshot.rs`, `state.rs`, `restore.rs`.
  - `src/input_method/`: `mod.rs`, `preset.rs`, `rule.rs`, `definitions.rs`.
  - `src/orthography/`: `mod.rs`, `phonetics.rs`, `spelling.rs`, `syllable.rs`.
  - `src/encoder/`: `mod.rs`, `tables.rs` (isolated 45 KB legacy charset definitions).
  - `src/dfa/`: `mod.rs`, `flattener.rs`.
- **100% Backward Compatibility:** All public API surfaces and `bamboo_core::advanced` re-exports preserved without breaking changes.
- **Clean Documentation & Clippy:** 0 Clippy warnings under strict lints, 8/8 rustdoc doctests passing, and clean `cargo doc` output.

## [0.3.23] - 2026-09-02

### Documentation & docs.rs
- **Comprehensive API Documentation:** Reached 100% rustdoc coverage under `#![warn(missing_docs)]` with zero warnings.
- **Enhanced Crate-Level Docs:** Added API selection guide, architecture overview, and complete doctests for real-time IME integration, 3-way diff, and dual backspace modes.
- **Detailed Module Docs:** Thorough documentation and runnable examples for `Config`, `ConfigBuilder`, `Dfa` (bitset/SWAR/arena internals), `encoder` (16 legacy charsets), and C-FFI safety contracts.
- **Docs.rs Metadata:** Configured `[package.metadata.docs.rs]` in `Cargo.toml`.

## [0.3.22] - 2026-08-24

### Safety & Code Quality
- **Strict Clippy Lints:** Added lint rules enforcing safety comments on unsafe blocks, doc backtick formatting, and avoiding anti-patterns (`clippy::undocumented_unsafe_blocks`, `clippy::doc_markdown`, `clippy::manual_let_else`, `clippy::semicolon_if_nothing_returned`, `clippy::match_same_arms`).
- **Documented Unsafe Blocks:** Added thorough `SAFETY` invariants documentation across DFA and flattener modules.
- **Contextual Invariant Checks:** Replaced bare `unwrap()` with explicit `.expect()` documentation in 8-byte word LCP comparison.
- **FFI Robustness:** Added safe mutex error handling preventing panics across FFI boundaries and added `#[repr(C)] pub enum BambooMethod``.

### Performance & Memory
- **Zero-Allocation Output (`Cow<str>`):** `Engine::output()` and `Engine::get_processed_str_cow()` return `Cow<'_, str>`, returning borrowed string slices for empty or uncommitted buffers without heap allocation.
- **Array-Based ASCII Rule Indexing:** Replaced `BTreeMap` key lookup in `Engine::with_config` with direct `[Vec<Rule>; 128]` array indexing, eliminating tree node allocations and O(log n) overhead.
- **Zero-Allocation Charset Name Iteration:** Added `charset_names() -> impl Iterator<Item = &'static str>` avoiding temporary `Vec<String>` allocations.
- **Static String Slices in Input Methods:** Changed `InputMethod::name` to `&'static str` avoiding heap strings for built-in methods.

### API & Design Improvements
- **Fluent Config Builder:** Added `ConfigBuilder` with fluent configuration methods (`Config::builder().free_tone_marking(...).build()`).
- **Semantic Enums:** Added `RestoreMark` enum (`RestoreMark::Yes`, `RestoreMark::No`) with seamless `From<bool>` / `Into<RestoreMark>` conversion for `remove_last_char`.
- **Public Getters:** Added accessor methods on `InputMethod` (`rules()`, `super_keys()`, `tone_keys()`, `appending_keys()`, `keys()`).

## [0.3.21] - 2026-08-19
