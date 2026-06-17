use leptos::prelude::*;
use wasm_bindgen::JsValue;

use crate::bridge::Bridge;
use crate::components::loader::Loader;
use crate::components::panel::Panel;
use crate::components::viewport::Viewport;
use crate::driver;
use crate::state::DemoState;

/// Application root: owns the shared state and bridge slot, composes the viewport
/// and the tool panel, and starts the demo timeline once the renderer is ready.
/// Falls back to a notice when the browser has no WebGPU.
#[component]
pub fn App() -> impl IntoView {
    if !webgpu_supported() {
        return unsupported().into_any();
    }

    let state = DemoState::new();
    let bridge = StoredValue::new_local(None::<Bridge>);
    let started = StoredValue::new(false);

    Effect::new(move |_| {
        if state.ready.get() && !started.get_value() {
            started.set_value(true);
            driver::start(state, bridge);
        }
    });

    view! {
        <div class="app-shell">
            <Viewport bridge state />
            <Panel bridge state />
            <Loader state />
        </div>
    }
    .into_any()
}

fn unsupported() -> impl IntoView {
    view! {
        <div class="unsupported">
            <div class="unsupported-card">
                <h1>"WebGPU not available"</h1>
                <p>
                    "This demo runs the Nightshade engine in a web worker through WebGPU. Open it in a browser with WebGPU and OffscreenCanvas-in-workers support (Chromium 113+, Firefox 141+)."
                </p>
            </div>
        </div>
    }
}

fn webgpu_supported() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Ok(navigator) = js_sys::Reflect::get(window.as_ref(), &JsValue::from_str("navigator"))
    else {
        return false;
    };
    js_sys::Reflect::get(&navigator, &JsValue::from_str("gpu"))
        .map(|gpu| !gpu.is_undefined() && !gpu.is_null())
        .unwrap_or(false)
}
