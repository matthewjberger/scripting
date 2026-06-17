use std::collections::HashSet;

use crate::state::Scene;
use nightshade::prelude::{Entity, World, despawn_recursive_immediate};
use nightshade_api::prelude::{clear_draw_pools, run_scripts, script_runtime_reset};
use protocol::WorkerMessage;

/// Pumps the script runtime once a frame. Clears the immediate-draw pools first
/// so a behavior script's `on_tick` draws (orbiting motes, lines) live for one
/// frame. A freshly installed snippet runs its `on_start` here; that is reported
/// once, on install. A persistent `on_tick` keeps producing commands every frame,
/// so success is not re-logged. Errors are logged when they change.
pub fn tick(scene: &mut Scene, world: &mut World) {
    if scene.reset_requested {
        scene.reset_requested = false;
        reset(scene, world);
        return;
    }

    clear_draw_pools(world);
    let report = run_scripts(world, &mut scene.runtime);

    if let Some(error) = report.errors.first() {
        if scene.last_error.as_deref() != Some(error.as_str()) {
            let mut message = error.clone();
            message.truncate(180);
            crate::post(&WorkerMessage::ScriptResult { ok: false, message });
            scene.last_error = Some(error.clone());
        }
        return;
    }

    scene.last_error = None;
    if scene.pending_ok {
        scene.pending_ok = false;
        crate::post(&WorkerMessage::ScriptResult {
            ok: true,
            message: String::new(),
        });
    }
}

/// Every live entity, for snapshotting the empty stage right after setup.
pub fn live_entities(world: &World) -> Vec<Entity> {
    let mut entities = Vec::new();
    world
        .core
        .query()
        .iter(|entity, _, _| entities.push(entity));
    entities
}

/// Despawns everything the snippets assembled, clears the script state, and
/// re-applies the plain stage, leaving the camera, sun, grid, and sky behind so
/// the scene can be built again from scratch.
pub fn reset(scene: &mut Scene, world: &mut World) {
    let keep: HashSet<Entity> = scene.base.iter().copied().collect();
    despawn_outside(world, &keep);
    nightshade::ecs::physics::systems::cleanup_physics_bodies_system(world);
    world
        .resources
        .entities
        .names
        .retain(|_, entity| world.core.get_name(*entity).is_some());
    world.resources.entities.tags.clear();
    world.resources.global_scripts.entries.clear();

    script_runtime_reset(&mut scene.runtime);
    scene.runtime.enabled = true;
    scene.serial = 0;
    scene.pending_ok = false;
    scene.last_error = None;

    crate::systems::setup::apply_stage(world);
    world.resources.mesh_render_state.request_full_rebuild();
    clear_draw_pools(world);
}

/// Despawn every live entity outside `keep`, skipping any a prior recursive
/// despawn already freed so the entity allocator is never double-freed.
fn despawn_outside(world: &mut World, keep: &HashSet<Entity>) {
    let mut current: Vec<Entity> = Vec::new();
    world.core.query().iter(|entity, _, _| current.push(entity));
    for entity in current {
        if keep.contains(&entity) {
            continue;
        }
        let alive = world
            .core
            .entity_locations
            .get(entity.id)
            .is_some_and(|location| location.allocated && location.generation == entity.generation);
        if alive {
            despawn_recursive_immediate(world, entity);
        }
    }
}
