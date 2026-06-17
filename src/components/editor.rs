use std::collections::HashSet;

use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// rhai keywords, colored as `tok-keyword`.
const KEYWORDS: &[&str] = &[
    "fn", "let", "const", "if", "else", "for", "in", "while", "loop", "return", "break",
    "continue", "switch", "import", "export", "global", "private", "true", "false", "throw", "try",
    "catch",
];

/// The api command methods (and the `commands` queue itself), colored as
/// `tok-command`. Highlighting only, so a generous list reads well without a
/// dependency on the engine's command manifest.
const COMMANDS: &[&str] = &[
    "commands",
    "spawn_floor",
    "spawn_object",
    "spawn_cube",
    "spawn_sphere",
    "spawn_cylinder",
    "spawn_cone",
    "spawn_plane",
    "spawn_torus",
    "spawn_label",
    "spawn_text",
    "point_light",
    "spot_light",
    "set_sun",
    "set_emissive",
    "set_color",
    "set_bloom",
    "set_metallic_roughness",
    "set_background",
    "set_ambient",
    "set_texture",
    "set_texture_tiling",
    "set_unlit",
    "draw_cube",
    "draw_sphere",
    "draw_line",
    "emit_firework",
    "emit_burst",
    "emit_particles",
    "emit_fire",
    "rotate",
    "set_position",
    "set_scale",
    "push",
    "set_velocity",
    "apply_force",
    "last",
    "result",
    "tag",
    "hsv",
    "rgb",
    "rgba",
    "random",
    "random_range",
    "random_int",
    "log",
];

/// Splits `source` into classed runs for the highlight layer. A hand-rolled
/// scanner over comments, strings, numbers, identifiers, and command calls, so
/// there is no JS highlighting dependency.
fn highlight(source: &str, commands: &HashSet<&'static str>) -> Vec<(&'static str, String)> {
    let chars: Vec<char> = source.chars().collect();
    let count = chars.len();
    let mut runs: Vec<(&'static str, String)> = Vec::new();
    let mut index = 0;
    while index < count {
        let current = chars[index];
        if current == '/' && index + 1 < count && chars[index + 1] == '/' {
            let start = index;
            while index < count && chars[index] != '\n' {
                index += 1;
            }
            runs.push(("tok-comment", chars[start..index].iter().collect()));
        } else if current == '"' {
            let start = index;
            index += 1;
            while index < count {
                if chars[index] == '\\' && index + 1 < count {
                    index += 2;
                    continue;
                }
                let quote = chars[index] == '"';
                index += 1;
                if quote {
                    break;
                }
            }
            runs.push(("tok-string", chars[start..index].iter().collect()));
        } else if current.is_ascii_digit() {
            let start = index;
            while index < count && (chars[index].is_ascii_digit() || chars[index] == '.') {
                index += 1;
            }
            runs.push(("tok-number", chars[start..index].iter().collect()));
        } else if current.is_alphabetic() || current == '_' {
            let start = index;
            while index < count && (chars[index].is_alphanumeric() || chars[index] == '_') {
                index += 1;
            }
            let word: String = chars[start..index].iter().collect();
            let class = if KEYWORDS.contains(&word.as_str()) {
                "tok-keyword"
            } else if commands.contains(word.as_str()) {
                "tok-command"
            } else {
                "tok-plain"
            };
            runs.push((class, word));
        } else {
            let start = index;
            index += 1;
            while index < count {
                let next = chars[index];
                let token_start = (next == '/' && index + 1 < count && chars[index + 1] == '/')
                    || next == '"'
                    || next.is_ascii_digit()
                    || next.is_alphabetic()
                    || next == '_';
                if token_start {
                    break;
                }
                index += 1;
            }
            runs.push(("tok-plain", chars[start..index].iter().collect()));
        }
    }
    runs
}

/// A rhai source view: a line-number gutter and a colored highlight layer behind
/// a textarea. While `editable` is false the textarea is read-only and a blinking
/// caret trails the highlighted text, so the timeline's typing reads as live
/// input. Once `editable` flips on, the textarea is the user's to edit and
/// `source` tracks their keystrokes.
#[component]
pub fn ScriptView(source: RwSignal<String>, editable: RwSignal<bool>) -> impl IntoView {
    let commands = StoredValue::new(COMMANDS.iter().copied().collect::<HashSet<&'static str>>());
    let layer = NodeRef::<leptos::html::Pre>::new();
    let gutter = NodeRef::<leptos::html::Div>::new();

    view! {
        <div class="script-editor-wrap">
            <div class="editor-gutter" node_ref=gutter>
                <div class="editor-gutter-inner">
                    {move || {
                        let count = source.with(|text| text.split('\n').count().max(1));
                        (1..=count).map(|line| line.to_string()).collect::<Vec<_>>().join("\n")
                    }}
                </div>
            </div>
            <div class="editor-body">
                <pre class="script-highlight" node_ref=layer aria-hidden="true">
                    {move || {
                        let text = source.get();
                        let runs = commands.with_value(|set| highlight(&text, set));
                        let spans = runs
                            .into_iter()
                            .map(|(class, run)| view! { <span class=class>{run}</span> })
                            .collect_view();
                        let caret = (!editable.get())
                            .then(|| view! { <span class="type-caret"></span> });
                        view! { {spans} {caret} }
                    }}
                </pre>
                <textarea
                    class="script-editor"
                    spellcheck="false"
                    prop:value=move || source.get()
                    prop:readonly=move || !editable.get()
                    on:input=move |event| source.set(event_target_value(&event))
                    on:scroll=move |event| {
                        if let Some(target) = event.target()
                            && let Ok(element) = target.dyn_into::<web_sys::HtmlElement>()
                        {
                            let top = element.scroll_top();
                            let left = element.scroll_left();
                            if let Some(layer) = layer.get() {
                                layer.set_scroll_top(top);
                                layer.set_scroll_left(left);
                            }
                            if let Some(gutter) = gutter.get() {
                                gutter.set_scroll_top(top);
                            }
                        }
                    }
                />
            </div>
        </div>
    }
}
