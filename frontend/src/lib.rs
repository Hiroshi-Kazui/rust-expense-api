// Library entry point — exposes modules for integration tests and wasm-bindgen-test.
// The [[bin]] target (src/main.rs) continues to use its own `mod` declarations.
pub mod error;
pub mod store;
pub mod types;
