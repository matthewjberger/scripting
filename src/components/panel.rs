use leptos::prelude::*;
use protocol::ClientMessage;
use wasm_bindgen::JsCast;

use crate::bridge::{Bridge, send};
use crate::components::editor::ScriptView;
use crate::driver;
use crate::state::DemoState;

/// The floating tool window: a step label and progress counter, the
/// syntax-highlighted script view the timeline types into, and the Back, Next,
/// and Run controls. Stepping is manual. After the last step the editor is the
/// user's to drive. The window sizes itself to the code it holds.
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
        }
    };

    let on_back = move |event: web_sys::MouseEvent| {
        blur(&event);
        driver::back(state, bridge);
    };

    let on_next = move |event: web_sys::MouseEvent| {
        blur(&event);
        driver::next(state);
    };

    let back_disabled = move || state.busy.get() || state.step.get() == 0;
    let next_disabled = move || state.busy.get() || state.step.get() + 1 >= driver::step_count();

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
                {move || {
                    let title = state.step_title.get();
                    (!title.is_empty()).then(|| view! { <div class="guide-title">{title}</div> })
                }}
            </div>

            <div class="editor-frame">
                <div class="editor-bar">
                    <span class="editor-name">"script.rhai"</span>
                    <div class="editor-actions">
                        <label class="autoplay">
                            <input
                                type="checkbox"
                                prop:checked=move || state.autoplay.get()
                                on:change=move |event| state.autoplay.set(event_target_checked(&event))
                            />
                            "Autoplay"
                        </label>
                        <button class="nav-button" prop:disabled=back_disabled on:click=on_back>
                            "Back"
                        </button>
                        <button class="nav-button" prop:disabled=next_disabled on:click=on_next>
                            "Next"
                        </button>
                        {move || {
                            state
                                .interactive
                                .get()
                                .then(|| {
                                    view! {
                                        <button class=run_class on:click=on_run>
                                            <span class="run-glyph">"\u{25B6}"</span>
                                            "Run"
                                        </button>
                                    }
                                })
                        }}
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
