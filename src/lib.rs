//! The Leptos UI on the main thread for the self-assembling scripting demo.
//!
//! The engine runs in a web worker on an `OffscreenCanvas`, driven through
//! `nightshade-api` and its rhai script runtime. This crate is only the page: a
//! syntax-highlighted script panel that types a snippet, "clicks" Run, and sends
//! the source to the worker, with a short step label for each. When the tour
//! ends the panel becomes a live editor.
//!
//! - `src/app.rs` composes the components and starts the demo timeline.
//! - `src/driver.rs` is the timeline: it types each snippet, flashes Run, and
//!   sends `RunScript` to the worker, then hands control to the user.
//! - `src/bridge.rs` spawns the worker and turns its messages into signal writes.
//! - `src/state.rs` is all page state, grouped as `Copy` signals.
//! - `src/components/` holds the viewport canvas, the editor, the panel, and the
//!   loader.

mod app;
mod bridge;
mod components;
mod driver;
mod state;

pub use app::App;
