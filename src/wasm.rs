//! WebAssembly (WASM) bindings for Bamboo Core.
//!
//! This module provides a high-level wrapper around the [`crate::Engine`] for use in Web browsers
//! and other WASM environments via `wasm-bindgen`.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// A WebAssembly-compatible wrapper for the Bamboo engine.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct BambooWasmEngine {
    inner: crate::engine::Engine,
}

#[cfg(feature = "wasm")]
impl Default for BambooWasmEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl BambooWasmEngine {
    /// Creates a new engine instance with the default Telex input method.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: crate::engine::Engine::from_preset(
                crate::input_method::InputMethodPreset::Telex,
            ),
        }
    }

    /// Processes a single character and returns the current transformed word.
    pub fn process_key(&mut self, key: char) -> String {
        self.inner.process_key(key, crate::mode::Mode::Vietnamese);
        self.inner.output().into_owned()
    }

    /// Resets the engine state, clearing all committed and active text.
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Returns the current transformed word output.
    pub fn output(&self) -> String {
        self.inner.output().into_owned()
    }

    /// Returns `true` when the most recent [`process_key`](Self::process_key)
    /// call finished the composition as a forced word boundary (the key did
    /// not fit the transformation buffer).
    ///
    /// The whole previous preedit was committed, so hosts must flush it to
    /// the application before continuing with [`output`](Self::output) as the
    /// new preedit.
    // wasm_bindgen rejects `const fn`; the inner call is a const fn but this
    // exported wrapper cannot be one.
    #[allow(clippy::missing_const_for_fn)]
    pub fn just_committed(&self) -> bool {
        self.inner.just_committed()
    }
}
