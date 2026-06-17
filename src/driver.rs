use std::time::Duration;

use leptos::prelude::*;
use protocol::ClientMessage;

use crate::bridge::{Bridge, send};
use crate::state::DemoState;

/// One step of the tour: a short label, the rhai snippet typed into the editor,
/// and how long to watch the scene before moving on.
struct Step {
    title: &'static str,
    code: &'static str,
    settle: f32,
}

const STEPS: &[Step] = &[
    Step {
        title: "Floor",
        settle: 1.2,
        code: "fn on_start() {\n    commands.spawn_floor(7.0);\n    commands.set_texture(result(0), \"proto_dark\");\n    commands.set_texture_tiling(result(0), 8.0);\n}\n",
    },
    Step {
        title: "Pillars",
        settle: 1.5,
        code: "fn on_start() {\n    let n = 8;\n    for i in 0..n {\n        let a = i / n.to_float() * tau;\n        commands.spawn_object(\"Cylinder\",\n            [a.cos() * 4.0, 1.3, a.sin() * 4.0],\n            [0.5, 2.6, 0.5], [1.0, 1.0, 1.0, 1.0], \"Static\");\n        let pillar = commands.last();\n        commands.set_texture(pillar, \"proto_light\");\n        commands.set_texture_tiling(pillar, 3.0);\n    }\n}\n",
    },
    Step {
        title: "Background",
        settle: 1.6,
        code: "fn on_start() {\n    commands.set_background(\"Nebula\");\n}\n",
    },
    Step {
        title: "Physics",
        settle: 2.6,
        code: "fn on_start() {\n    for i in 0..6 {\n        commands.spawn_object(\"Sphere\",\n            [random_range(-2.5, 2.5), 7.0 + i, random_range(-2.5, 2.5)],\n            [1.0, 1.0, 1.0], hsv(i / 6.0, 0.7, 1.0),\n            #{ Dynamic: #{ mass: 1.0 } });\n    }\n}\n",
    },
    Step {
        title: "Lights",
        settle: 1.6,
        code: "fn on_start() {\n    let lights = [\n        [0.0, 5.0, 0.0, 1.0, 0.85, 0.5],\n        [3.6, 3.0, 3.6, 0.4, 0.6, 1.0],\n        [-3.6, 3.0, -3.6, 1.0, 0.4, 0.6],\n    ];\n    for p in lights {\n        commands.point_light([p[0], p[1], p[2]], [p[3], p[4], p[5]], 45.0);\n        commands.spawn_object(\"Sphere\", [p[0], p[1], p[2]],\n            [0.3, 0.3, 0.3], [p[3], p[4], p[5], 1.0], \"None\");\n        let mark = commands.last();\n        commands.set_emissive(mark, [p[3], p[4], p[5]], 6.0);\n        commands.set_unlit(mark, true);\n    }\n}\n",
    },
    Step {
        title: "Torus",
        settle: 1.7,
        code: "fn on_start() {\n    commands.spawn_object(\"Cylinder\", [0.0, 0.5, 0.0],\n        [1.2, 1.0, 1.2], [1.0, 1.0, 1.0, 1.0], \"Static\");\n    let pedestal = commands.last();\n    commands.set_texture(pedestal, \"proto_orange\");\n    commands.set_texture_tiling(pedestal, 2.0);\n    commands.spawn_object(\"Torus\", [0.0, 2.2, 0.0],\n        [1.1, 1.1, 1.1], [1.0, 0.55, 0.2, 1.0], \"None\");\n    commands.set_emissive(commands.last(), [1.0, 0.45, 0.12], 3.0);\n}\n",
    },
    Step {
        title: "Metallic spheres",
        settle: 3.0,
        code: "fn on_start() {\n    state.motes = [];\n    for i in 0..5 {\n        commands.spawn_object(\"Sphere\", [2.8, 2.6, 0.0],\n            [0.45, 0.45, 0.45], [0.95, 0.96, 1.0, 1.0], \"None\");\n        commands.tag(\"mote\" + i);\n    }\n}\n\nfn on_tick() {\n    let n = 5;\n    if state.motes.len() < n {\n        for i in 0..n {\n            let key = \"mote\" + i;\n            if (key in replies) && (state.motes.len() == i) {\n                state.motes.push(replies[key]);\n                commands.set_metallic_roughness(replies[key], 1.0, 0.08);\n            }\n        }\n    }\n    for i in 0..state.motes.len() {\n        let a = time + i / n.to_float() * tau;\n        let y = 2.6 + (time * 2.0 + i).sin() * 0.5;\n        commands.set_position(state.motes[i],\n            [a.cos() * 2.8, y, a.sin() * 2.8]);\n    }\n}\n",
    },
    Step {
        title: "Helix",
        settle: 3.0,
        code: "fn on_start() {\n    state.helix = [];\n    for i in 0..12 {\n        commands.spawn_object(\"Sphere\", [0.0, 2.0, 0.0],\n            [0.25, 0.25, 0.25], [1.0, 1.0, 1.0, 1.0], \"None\");\n        commands.tag(\"h\" + i);\n    }\n}\n\nfn on_tick() {\n    let n = 12;\n    if state.helix.len() < n {\n        for i in 0..n {\n            let key = \"h\" + i;\n            if (key in replies) && (state.helix.len() == i) {\n                state.helix.push(replies[key]);\n                let c = hsv(i / n.to_float(), 0.85, 1.0);\n                commands.set_emissive(replies[key], [c[0], c[1], c[2]], 4.0);\n            }\n        }\n    }\n    for i in 0..state.helix.len() {\n        let t = i / n.to_float();\n        let a = time * 1.5 + t * tau * 2.0;\n        commands.set_position(state.helix[i],\n            [a.cos() * 1.7, 1.4 + t * 4.0, a.sin() * 1.7]);\n    }\n}\n",
    },
    Step {
        title: "Ripple grid",
        settle: 2.4,
        code: "fn on_tick() {\n    let s = 7;\n    for x in 0..s {\n        for z in 0..s {\n            let fx = x - 3;\n            let fz = z - 3;\n            let d = (fx * fx + fz * fz).to_float().sqrt();\n            let h = 5.5 + (time * 2.5 - d).sin() * 0.6;\n            commands.draw_cube([fx * 1.3, h, fz * 1.3],\n                [0.3, 0.3, 0.3], hsv((d * 0.12 + time * 0.15) % 1.0, 0.7, 1.0));\n        }\n    }\n}\n",
    },
    Step {
        title: "Fireworks",
        settle: 3.5,
        code: "fn on_start() {\n    state.fw = 0.0;\n}\n\nfn on_tick() {\n    state.fw += dt;\n    if state.fw > 0.6 {\n        state.fw = 0.0;\n        commands.emit_firework(\n            [random_range(-3.0, 3.0), 0.5, random_range(-3.0, 3.0)],\n            [random_range(-1.5, 1.5), random_range(8.0, 11.0), random_range(-1.5, 1.5)]);\n    }\n}\n",
    },
];

const TICK_MS: u64 = 24;
const TICK: f32 = TICK_MS as f32 / 1000.0;
const INTRO: f32 = 1.8;
const NARRATE: f32 = 1.0;
const HOLD: f32 = 0.45;
const CLICK: f32 = 0.5;
const CHARS_PER_TICK: usize = 2;

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Intro,
    Narrate,
    Typing,
    Hold,
    Click,
    Settle,
    Done,
}

#[derive(Clone, Copy)]
struct Timeline {
    step: usize,
    phase: Phase,
    timer: f32,
    typed: usize,
}

/// Starts the tour once the renderer is ready: shows the intro, then ticks a
/// small state machine on an interval that types each snippet, flashes Run, and
/// sends it to the worker. The scene keeps every script it installs, so each step
/// layers on more behavior. After the last step the editor goes live.
pub fn start(state: DemoState, bridge: StoredValue<Option<Bridge>, LocalStorage>) {
    let timeline = StoredValue::new(Timeline {
        step: 0,
        phase: Phase::Intro,
        timer: INTRO,
        typed: 0,
    });

    let advance = move || {
        timeline.update_value(|tl| step(tl, state, bridge));
    };

    // The handle is `Copy` and does not clear the interval when dropped, so
    // ignoring it leaves the timeline ticking for the life of the page.
    let _ = set_interval_with_handle(advance, Duration::from_millis(TICK_MS));
}

fn enter_narrate(tl: &mut Timeline, state: DemoState, step_index: usize) {
    let step = &STEPS[step_index];
    state.step_title.set(step.title.to_string());
    state
        .progress
        .set(format!("Step {} / {}", step_index + 1, STEPS.len()));
    state.code.set(String::new());
    tl.step = step_index;
    tl.typed = 0;
    tl.phase = Phase::Narrate;
    tl.timer = NARRATE;
}

fn enter_interactive(tl: &mut Timeline, state: DemoState) {
    state.interactive.set(true);
    state.step_title.set(String::new());
    state.progress.set(String::new());
    tl.phase = Phase::Done;
}

fn step(tl: &mut Timeline, state: DemoState, bridge: StoredValue<Option<Bridge>, LocalStorage>) {
    if state.aborted.get_untracked() {
        tl.phase = Phase::Done;
        return;
    }
    match tl.phase {
        Phase::Intro => {
            tl.timer -= TICK;
            if tl.timer <= 0.0 {
                enter_narrate(tl, state, 0);
            }
        }
        Phase::Narrate => {
            tl.timer -= TICK;
            if tl.timer <= 0.0 {
                tl.phase = Phase::Typing;
            }
        }
        Phase::Typing => {
            let code = STEPS[tl.step].code;
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
                if let Some(bridge) = bridge.get_value() {
                    send(
                        &bridge,
                        &ClientMessage::RunScript {
                            source: STEPS[tl.step].code.to_string(),
                        },
                    );
                }
                state.running.set(false);
                tl.phase = Phase::Settle;
                tl.timer = STEPS[tl.step].settle;
            }
        }
        Phase::Settle => {
            tl.timer -= TICK;
            if tl.timer <= 0.0 {
                let next = tl.step + 1;
                if next < STEPS.len() {
                    enter_narrate(tl, state, next);
                } else {
                    enter_interactive(tl, state);
                }
            }
        }
        Phase::Done => {}
    }
}
