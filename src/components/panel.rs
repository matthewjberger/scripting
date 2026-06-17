use leptos::prelude::*;
use protocol::ClientMessage;
use wasm_bindgen::JsCast;

use crate::bridge::{Bridge, send};
use crate::components::editor::ScriptView;
use crate::state::DemoState;

/// The floating tool window: a guide that narrates the current step, the
/// syntax-highlighted script view the timeline types into, and the Run control.
/// After the tour the editor is the user's to drive. The window sizes itself to
/// the code it holds.
#[component]
pub fn Panel(bridge: StoredValue<Option<Bridge>, LocalStorage>, state: DemoState) -> impl IntoView {
    let blur = |event: &web_sys::MouseEvent| {
        if let Some(button) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let _ = button.blur();
        }
    };

    let on_run = move |event: web_sys::MouseEvent| {
        blur(&event);
        if !state.interactive.get_untracked() {
            return;
        }
        if let Some(bridge) = bridge.get_value() {
            send(
                &bridge,
                &ClientMessage::RunScript {
                    source: state.code.get_untracked(),
                },
            );
            state.step_title.set("Running".to_string());
            state
                .step_blurb
                .set("Your script is live. Edit and Run again to rebuild.".to_string());
            state.progress.set(String::new());
        }
    };

    let run_class = move || {
        if state.running.get() {
            "run-button flash"
        } else {
            "run-button"
        }
    };

    view! {
        <div class="panel">
            <div class="panel-head">
                <span class="panel-dot"></span>
                <span class="panel-title">"self assembling"</span>
                <span class="panel-sub">
                    {move || format!("{} obj · {:.0} fps", state.entity_count.get(), state.fps.get())}
                </span>
            </div>

            <div class="guide">
                {move || {
                    let progress = state.progress.get();
                    (!progress.is_empty())
                        .then(|| view! { <div class="guide-step">{progress}</div> })
                }}
                <div class="guide-title">{move || state.step_title.get()}</div>
                <div class="guide-blurb">{move || state.step_blurb.get()}</div>
            </div>

            <div class="editor-frame">
                <div class="editor-bar">
                    <span class="editor-name">"script.rhai"</span>
                    <div class="editor-actions">
                        <button class=run_class on:click=on_run>
                            <span class="run-glyph">"\u{25B6}"</span>
                            "Run"
                        </button>
                    </div>
                </div>
                <ScriptView source=state.code editable=state.interactive />
            </div>

            {move || {
                let error = state.error.get();
                (!error.is_empty()).then(|| view! { <div class="error-line">{error}</div> })
            }}
        </div>
    }
}
