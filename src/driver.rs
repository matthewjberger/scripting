use std::time::Duration;

use leptos::prelude::*;
use protocol::ClientMessage;

use crate::bridge::{Bridge, send};
use crate::state::DemoState;

/// One step of the tour: a short label and the rhai snippet typed into the
/// editor when the step is first reached.
struct Step {
    title: &'static str,
    code: &'static str,
}

const STEPS: &[Step] = &[
    Step {
        title: "Floor",
        code: "fn on_start() {\n    commands.spawn_floor(7.0);\n    commands.set_texture(result(0), \"proto_dark\");\n    commands.set_texture_tiling(result(0), 8.0);\n}\n",
    },
    Step {
        title: "Pillars",
        code: "fn on_start() {\n    let n = 8;\n    for i in 0..n {\n        let a = i / n.to_float() * tau;\n        commands.spawn_object(\"Cylinder\",\n            [a.cos() * 4.0, 1.3, a.sin() * 4.0],\n            [0.5, 2.6, 0.5], [1.0, 1.0, 1.0, 1.0], \"Static\");\n        let pillar = commands.last();\n        commands.set_texture(pillar, \"proto_light\");\n        commands.set_texture_tiling(pillar, 3.0);\n    }\n}\n",
    },
    Step {
        title: "Background",
        code: "fn on_start() {\n    commands.set_background(\"Nebula\");\n}\n",
    },
    Step {
        title: "Physics",
        code: "fn on_start() {\n    for i in 0..6 {\n        commands.spawn_object(\"Sphere\",\n            [random_range(-2.5, 2.5), 7.0 + i, random_range(-2.5, 2.5)],\n            [1.0, 1.0, 1.0], hsv(i / 6.0, 0.7, 1.0),\n            #{ Dynamic: #{ mass: 1.0 } });\n    }\n}\n",
    },
    Step {
        title: "Lights",
        code: "fn on_start() {\n    let lights = [\n        [0.0, 5.0, 0.0, 1.0, 0.85, 0.5],\n        [3.6, 3.0, 3.6, 0.4, 0.6, 1.0],\n        [-3.6, 3.0, -3.6, 1.0, 0.4, 0.6],\n    ];\n    for p in lights {\n        commands.point_light([p[0], p[1], p[2]], [p[3], p[4], p[5]], 45.0);\n        commands.spawn_object(\"Sphere\", [p[0], p[1], p[2]],\n            [0.3, 0.3, 0.3], [p[3], p[4], p[5], 1.0], \"None\");\n        let mark = commands.last();\n        commands.set_emissive(mark, [p[3], p[4], p[5]], 6.0);\n        commands.set_unlit(mark, true);\n    }\n}\n",
    },
    Step {
        title: "Torus",
        code: "fn on_start() {\n    commands.spawn_object(\"Cylinder\", [0.0, 0.5, 0.0],\n        [1.2, 1.0, 1.2], [1.0, 1.0, 1.0, 1.0], \"Static\");\n    let pedestal = commands.last();\n    commands.set_texture(pedestal, \"proto_orange\");\n    commands.set_texture_tiling(pedestal, 2.0);\n    commands.spawn_object(\"Torus\", [0.0, 2.2, 0.0],\n        [1.1, 1.1, 1.1], [1.0, 0.55, 0.2, 1.0], \"None\");\n    commands.set_emissive(commands.last(), [1.0, 0.45, 0.12], 3.0);\n}\n",
    },
    Step {
        title: "Metallic spheres",
        code: "fn on_start() {\n    state.motes = [];\n    for i in 0..5 {\n        commands.spawn_object(\"Sphere\", [2.8, 2.6, 0.0],\n            [0.45, 0.45, 0.45], [0.95, 0.96, 1.0, 1.0], \"None\");\n        commands.tag(\"mote\" + i);\n    }\n}\n\nfn on_tick() {\n    let n = 5;\n    if state.motes.len() < n {\n        for i in 0..n {\n            let key = \"mote\" + i;\n            if (key in replies) && (state.motes.len() == i) {\n                state.motes.push(replies[key]);\n                commands.set_metallic_roughness(replies[key], 1.0, 0.08);\n            }\n        }\n    }\n    for i in 0..state.motes.len() {\n        let a = time + i / n.to_float() * tau;\n        let y = 2.6 + (time * 2.0 + i).sin() * 0.5;\n        commands.set_position(state.motes[i],\n            [a.cos() * 2.8, y, a.sin() * 2.8]);\n    }\n}\n",
    },
    Step {
        title: "Helix",
        code: "fn on_start() {\n    state.helix = [];\n    for i in 0..12 {\n        commands.spawn_object(\"Sphere\", [0.0, 2.0, 0.0],\n            [0.25, 0.25, 0.25], [1.0, 1.0, 1.0, 1.0], \"None\");\n        commands.tag(\"h\" + i);\n    }\n}\n\nfn on_tick() {\n    let n = 12;\n    if state.helix.len() < n {\n        for i in 0..n {\n            let key = \"h\" + i;\n            if (key in replies) && (state.helix.len() == i) {\n                state.helix.push(replies[key]);\n                let c = hsv(i / n.to_float(), 0.85, 1.0);\n                commands.set_emissive(replies[key], [c[0], c[1], c[2]], 4.0);\n            }\n        }\n    }\n    for i in 0..state.helix.len() {\n        let t = i / n.to_float();\n        let a = time * 1.5 + t * tau * 2.0;\n        commands.set_position(state.helix[i],\n            [a.cos() * 1.7, 1.4 + t * 4.0, a.sin() * 1.7]);\n    }\n}\n",
    },
    Step {
        title: "Ripple grid",
        code: "fn on_tick() {\n    let s = 7;\n    for x in 0..s {\n        for z in 0..s {\n            let fx = x - 3;\n            let fz = z - 3;\n            let d = (fx * fx + fz * fz).to_float().sqrt();\n            let h = 5.5 + (time * 2.5 - d).sin() * 0.6;\n            commands.draw_cube([fx * 1.3, h, fz * 1.3],\n                [0.3, 0.3, 0.3], hsv((d * 0.12 + time * 0.15) % 1.0, 0.7, 1.0));\n        }\n    }\n}\n",
    },
    Step {
        title: "Fireworks",
        code: "fn on_start() {\n    state.fw = 0.0;\n}\n\nfn on_tick() {\n    state.fw += dt;\n    if state.fw > 0.6 {\n        state.fw = 0.0;\n        commands.emit_firework(\n            [random_range(-3.0, 3.0), 0.5, random_range(-3.0, 3.0)],\n            [random_range(-1.5, 1.5), random_range(8.0, 11.0), random_range(-1.5, 1.5)]);\n    }\n}\n",
    },
];

const TICK_MS: u64 = 24;
const TICK: f32 = TICK_MS as f32 / 1000.0;
const HOLD: f32 = 0.45;
const CLICK: f32 = 0.5;
const SETTLE: f32 = 1.6;
const CHARS_PER_TICK: usize = 2;

/// The number of steps in the tour.
pub fn step_count() -> usize {
    STEPS.len()
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Idle,
    Typing,
    Hold,
    Click,
    Settle,
}

#[derive(Clone, Copy)]
struct Timeline {
    phase: Phase,
    timer: f32,
    typed: usize,
}

/// Starts the tour on the first step and runs the typing loop. With autoplay on
/// (the default) it advances on its own; unchecking it pauses so the user can
/// step with [`next`] and [`back`]. Forward adds the next script; back rebuilds
/// the scene with every script up to the target.
pub fn start(state: DemoState, bridge: StoredValue<Option<Bridge>, LocalStorage>) {
    state.step.set(0);
    set_meta(state, 0);
    state.busy.set(true);

    let timeline = StoredValue::new(Timeline {
        phase: Phase::Idle,
        timer: 0.0,
        typed: 0,
    });
    let advance = move || {
        timeline.update_value(|tl| tick(tl, state, bridge));
    };

    // The handle is `Copy` and does not clear the interval when dropped, so
    // ignoring it leaves the typing loop running for the life of the page.
    let _ = set_interval_with_handle(advance, Duration::from_millis(TICK_MS));
}

/// Advances to the next step, typing and running its snippet on top of the
/// current scene. Ignored while a step is running or at the last step.
pub fn next(state: DemoState) {
    if state.busy.get_untracked() {
        return;
    }
    let next = state.step.get_untracked() + 1;
    if next >= STEPS.len() {
        return;
    }
    state.step.set(next);
    set_meta(state, next);
    state.busy.set(true);
}

/// Goes back a step. Turns off autoplay and rebuilds the scene from scratch with
/// every script up to and including the previous step, so it shows the scene as
/// it was then. Ignored while a step is running or at the first step.
pub fn back(state: DemoState, bridge: StoredValue<Option<Bridge>, LocalStorage>) {
    if state.busy.get_untracked() {
        return;
    }
    let current = state.step.get_untracked();
    if current == 0 {
        return;
    }
    state.autoplay.set(false);
    let previous = current - 1;
    state.step.set(previous);
    set_meta(state, previous);
    state.code.set(STEPS[previous].code.to_string());
    if let Some(bridge) = bridge.get_value() {
        let sources = STEPS[..=previous]
            .iter()
            .map(|step| step.code.to_string())
            .collect();
        send(&bridge, &ClientMessage::ApplyScripts { sources });
    }
}

fn set_meta(state: DemoState, index: usize) {
    state.step_title.set(STEPS[index].title.to_string());
    state
        .progress
        .set(format!("Step {} / {}", index + 1, STEPS.len()));
}

/// One interval tick of the typing loop. A step types in full, then runs once.
/// After a step, autoplay waits then advances; otherwise it stops until the user
/// steps.
fn tick(tl: &mut Timeline, state: DemoState, bridge: StoredValue<Option<Bridge>, LocalStorage>) {
    match tl.phase {
        Phase::Idle => {
            if state.busy.get_untracked() {
                tl.typed = 0;
                tl.phase = Phase::Typing;
            } else if state.autoplay.get_untracked() && state.step.get_untracked() + 1 < STEPS.len()
            {
                state.busy.set(true);
                tl.phase = Phase::Settle;
                tl.timer = SETTLE;
            }
        }
        Phase::Typing => {
            let code = STEPS[state.step.get_untracked()].code;
            let total = code.chars().count();
            tl.typed = (tl.typed + CHARS_PER_TICK).min(total);
            let shown: String = code.chars().take(tl.typed).collect();
            state.code.set(shown);
            if tl.typed >= total {
                tl.phase = Phase::Hold;
                tl.timer = HOLD;
            }
        }
        Phase::Hold => {
            tl.timer -= TICK;
            if tl.timer <= 0.0 {
                tl.phase = Phase::Click;
                tl.timer = CLICK;
                state.running.set(true);
            }
        }
        Phase::Click => {
            tl.timer -= TICK;
            if tl.timer <= 0.0 {
                let index = state.step.get_untracked();
                if let Some(bridge) = bridge.get_value() {
                    send(
                        &bridge,
                        &ClientMessage::RunScript {
                            source: STEPS[index].code.to_string(),
                        },
                    );
                }
                state.running.set(false);
                if index + 1 == STEPS.len() {
                    state.interactive.set(true);
                }
                if state.autoplay.get_untracked() && index + 1 < STEPS.len() {
                    tl.phase = Phase::Settle;
                    tl.timer = SETTLE;
                } else {
                    state.busy.set(false);
                    tl.phase = Phase::Idle;
                }
            }
        }
        Phase::Settle => {
            if !state.autoplay.get_untracked() {
                state.busy.set(false);
                tl.phase = Phase::Idle;
                return;
            }
            tl.timer -= TICK;
            if tl.timer <= 0.0 {
                let next = state.step.get_untracked() + 1;
                state.step.set(next);
                set_meta(state, next);
                tl.typed = 0;
                tl.phase = Phase::Typing;
            }
        }
    }
}
