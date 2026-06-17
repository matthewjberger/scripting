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
    /// Set when the user presses Reset, so the tour timeline stops installing
    /// further steps instead of rebuilding the scene it just cleared.
    pub aborted: RwSignal<bool>,

    /// The script source shown in the editor. The driver writes it character by
    /// character during the tour; the user edits it afterward.
    pub code: RwSignal<String>,
    pub step_title: RwSignal<String>,
    pub step_blurb: RwSignal<String>,
    pub progress: RwSignal<String>,
    /// True while the Run button is being "pressed" by the timeline.
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
            aborted: RwSignal::new(false),
            code: RwSignal::new(String::new()),
            step_title: RwSignal::new(String::new()),
            step_blurb: RwSignal::new(String::new()),
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
