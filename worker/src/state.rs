use crate::systems;
use nightshade::prelude::*;
use nightshade_api::prelude::ScriptRuntime;

/// The application state carried across frames by the offscreen driver. It owns
/// the script runtime that compiles and runs the snippets the page sends, plus a
/// serial used to give each installed snippet a fresh name so its `on_start`
/// runs exactly once.
pub struct Scene {
    pub runtime: ScriptRuntime,
    pub serial: usize,
    /// The entities present after `initialize` (camera, sun): everything else is
    /// script-spawned and a reset despawns it.
    pub base: Vec<Entity>,
    /// Set when a snippet is installed, so the next tick reports its run once.
    /// Persistent `on_tick` scripts produce commands every frame, so success is
    /// reported on install, not per frame.
    pub pending_ok: bool,
    /// The last error surfaced, to avoid logging a failing script every frame.
    pub last_error: Option<String>,
    /// Set by a `ResetScene` message; the next tick performs the reset inside the
    /// frame so the mesh rebuild lands cleanly, the way the editor does it.
    pub reset_requested: bool,
    /// Set by an `ApplyScripts` message; the next tick resets the scene and
    /// installs these snippets in order, so navigation can jump to any step.
    pub pending_apply: Option<Vec<String>>,
}

impl Scene {
    pub fn new() -> Self {
        let mut runtime = ScriptRuntime::default();
        runtime.enabled = true;
        Self {
            runtime,
            serial: 0,
            base: Vec::new(),
            pending_ok: false,
            last_error: None,
            reset_requested: false,
            pending_apply: None,
        }
    }
}

impl State for Scene {
    fn initialize(&mut self, world: &mut World) {
        systems::setup::initialize(world);
    }

    fn run_systems(&mut self, world: &mut World) {
        camera_controllers_system(world);
        systems::scene::tick(self, world);
    }
}
