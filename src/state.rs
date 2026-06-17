use leptos::prelude::*;

/// All page state, grouped as signals. `Copy`, so it threads into every
/// component and closure without cloning.
#[derive(Clone, Copy)]
pub struct DemoState {
    pub ready: RwSignal<bool>,
    pub adapter: RwSignal<String>,
    pub fps: RwSignal<f32>,
    pub entity_count: RwSignal<u32>,
    pub grabbing: RwSignal<bool>,

    /// The step currently shown in the panel and reflected in the scene.
    /// Forward adds the next script; back rebuilds a fresh scene with every
    /// script up to and including the target.
    pub step: RwSignal<usize>,
    /// True while a step is typing or running, so the controls are disabled.
    pub busy: RwSignal<bool>,
    /// When set, the tour advances on its own. Unchecking it pauses after the
    /// current step so the user can step with Back and Next. Defaults to on.
    pub autoplay: RwSignal<bool>,

    /// The script source shown in the editor. The driver writes it character by
    /// character when a step first runs; the user edits it afterward.
    pub code: RwSignal<String>,
    pub step_title: RwSignal<String>,
    pub progress: RwSignal<String>,
    /// True for a brief flash while a step is being run.
    pub running: RwSignal<bool>,
    /// True once the tour finishes and the editor is the user's to drive.
    pub interactive: RwSignal<bool>,
    /// The current script error, shown under the editor, or empty.
    pub error: RwSignal<String>,
}

impl DemoState {
    pub fn new() -> Self {
        Self {
            ready: RwSignal::new(false),
            adapter: RwSignal::new(String::new()),
            fps: RwSignal::new(0.0),
            entity_count: RwSignal::new(0),
            grabbing: RwSignal::new(false),
            step: RwSignal::new(0),
            busy: RwSignal::new(false),
            autoplay: RwSignal::new(true),
            code: RwSignal::new(String::new()),
            step_title: RwSignal::new(String::new()),
            progress: RwSignal::new(String::new()),
            running: RwSignal::new(false),
            interactive: RwSignal::new(false),
            error: RwSignal::new(String::new()),
        }
    }
}

impl Default for DemoState {
    fn default() -> Self {
        Self::new()
    }
}
