//! The wasm module inside the web worker. Owns the `OffscreenCanvas`, the engine
//! `World`, the script runtime, and the render loop. The page talks to it only
//! through the `protocol` messages.
//!
//! The scene is assembled at runtime by rhai snippets the page sends: each
//! `RunScript` installs a global script whose `on_start` runs once and pushes
//! commands (spawn a floor, a ring of pillars, falling spheres). The worker drops
//! to the raw engine only for the parts the facade does not cover from a worker:
//! the `OffscreenCanvas` renderer, the offscreen frame driver, and input
//! injection.

mod state;
mod systems;

use std::cell::RefCell;
use std::rc::Rc;

use nightshade::ecs::script::components::GlobalScript;
use nightshade::prelude::*;
use nightshade::render::wgpu::create_wgpu_renderer;
use protocol::{CANVAS_KEY, ClientMessage, MESSAGE_KEY, WorkerMessage};
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, OffscreenCanvas};

use crate::state::Scene;

type AppSlot = Rc<RefCell<Option<App>>>;
type PendingMessages = Rc<RefCell<Vec<JsValue>>>;

struct App {
    world: World,
    renderer: WgpuRenderer,
    state: Scene,
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
    let app_slot: AppSlot = Rc::new(RefCell::new(None));
    let pending: PendingMessages = Rc::new(RefCell::new(Vec::new()));

    let handler_scope = scope.clone();
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        handle_data(&handler_scope, &app_slot, &pending, event.data());
    });
    scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();
}

fn handle_data(
    scope: &DedicatedWorkerGlobalScope,
    app_slot: &AppSlot,
    pending: &PendingMessages,
    data: JsValue,
) {
    let Ok(payload) = js_sys::Reflect::get(&data, &JsValue::from_str(MESSAGE_KEY)) else {
        return;
    };
    let Ok(message) = serde_wasm_bindgen::from_value::<ClientMessage>(payload) else {
        return;
    };

    if !matches!(message, ClientMessage::Init { .. }) && app_slot.borrow().is_none() {
        pending.borrow_mut().push(data);
        return;
    }

    match message {
        ClientMessage::Init { width, height } => {
            let Some(canvas) = canvas_from(&data) else {
                return;
            };
            let scope = scope.clone();
            let app_slot = app_slot.clone();
            let pending = pending.clone();
            spawn_local(async move {
                let app = create_app(canvas, width, height).await;
                *app_slot.borrow_mut() = Some(app);
                let queued = std::mem::take(&mut *pending.borrow_mut());
                for data in queued {
                    handle_data(&scope, &app_slot, &pending, data);
                }
                post(&WorkerMessage::Ready {
                    adapter: "WebGPU".to_string(),
                });
                start_render_loop(app_slot);
            });
        }
        ClientMessage::Resize { width, height } => {
            if let Some(app) = app_slot.borrow_mut().as_mut() {
                let physical_width = (width as u32).max(1);
                let physical_height = (height as u32).max(1);
                resize_offscreen(
                    &mut app.world,
                    &mut app.renderer,
                    physical_width,
                    physical_height,
                );
                app.world.resources.window.active_viewport_rect =
                    Some(nightshade::ecs::window::resources::ViewportRect {
                        x: 0.0,
                        y: 0.0,
                        width: physical_width as f32,
                        height: physical_height as f32,
                    });
            }
        }
        other => {
            if let Some(app) = app_slot.borrow_mut().as_mut() {
                apply_client_message(&mut app.world, &mut app.state, other);
            }
        }
    }
}

fn apply_client_message(world: &mut World, scene: &mut Scene, message: ClientMessage) {
    match message {
        ClientMessage::PointerMove { x, y } => {
            input_inject_cursor_moved(world, Vec2::new(x, y));
        }
        ClientMessage::PointerButton { button, pressed } => {
            let state = if pressed {
                KeyState::Pressed
            } else {
                KeyState::Released
            };
            input_inject_mouse_button(world, mouse_button(button), state);
        }
        ClientMessage::Wheel { delta } => {
            input_inject_mouse_wheel(world, Vec2::new(0.0, -delta / 100.0));
        }
        ClientMessage::Touch { id, phase, x, y } => {
            input_inject_touch(world, id, touch_phase(phase), Vec2::new(x, y));
        }
        ClientMessage::RunScript { source } => {
            scene.serial += 1;
            scene.pending_ok = true;
            world.resources.global_scripts.entries.push(GlobalScript {
                name: format!("step_{}", scene.serial),
                source,
                enabled: true,
            });
        }
        ClientMessage::ResetScene => {
            scene.reset_requested = true;
        }
        ClientMessage::Init { .. } | ClientMessage::Resize { .. } => {}
    }
}

async fn create_app(canvas: OffscreenCanvas, width: f32, height: f32) -> App {
    let physical_width = (width as u32).max(1);
    let physical_height = (height as u32).max(1);

    let surface_target = wgpu::SurfaceTarget::OffscreenCanvas(canvas);
    let mut renderer = create_wgpu_renderer(surface_target, physical_width, physical_height)
        .await
        .expect("failed to create renderer from offscreen canvas");

    let mut world = World::default();
    let mut state = Scene::new();
    initialize_offscreen(
        &mut world,
        &mut state,
        &mut renderer,
        (physical_width, physical_height),
        1.0,
    );
    world.resources.window.active_viewport_rect =
        Some(nightshade::ecs::window::resources::ViewportRect {
            x: 0.0,
            y: 0.0,
            width: physical_width as f32,
            height: physical_height as f32,
        });

    state.base = systems::scene::live_entities(&world);

    App {
        world,
        renderer,
        state,
    }
}

fn start_render_loop(app_slot: AppSlot) {
    let last_push = Rc::new(RefCell::new(0.0_f64));

    spawn_animation_frame_loop(move || {
        if let Some(app) = app_slot.borrow_mut().as_mut() {
            tick_offscreen(&mut app.world, &mut app.state, &mut app.renderer);
            let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
            if let Some(performance) = scope.performance() {
                let now = performance.now();
                let mut last = last_push.borrow_mut();
                if now - *last > 500.0 {
                    *last = now;
                    let entity_count = app
                        .world
                        .core
                        .query_entities(
                            nightshade::ecs::world::LOCAL_TRANSFORM
                                | nightshade::ecs::world::GLOBAL_TRANSFORM,
                        )
                        .count() as u32;
                    post(&WorkerMessage::Stats {
                        fps: app.world.resources.window.timing.frames_per_second,
                        entity_count,
                    });
                }
            }
        }
    });
}

fn mouse_button(button: u8) -> MouseButton {
    match button {
        1 => MouseButton::Middle,
        2 => MouseButton::Right,
        _ => MouseButton::Left,
    }
}

fn touch_phase(phase: protocol::TouchPhase) -> TouchPhase {
    match phase {
        protocol::TouchPhase::Started => TouchPhase::Started,
        protocol::TouchPhase::Moved => TouchPhase::Moved,
        protocol::TouchPhase::Ended => TouchPhase::Ended,
        protocol::TouchPhase::Cancelled => TouchPhase::Cancelled,
    }
}

fn canvas_from(data: &JsValue) -> Option<OffscreenCanvas> {
    js_sys::Reflect::get(data, &JsValue::from_str(CANVAS_KEY))
        .ok()
        .and_then(|value| value.dyn_into::<OffscreenCanvas>().ok())
}

pub(crate) fn post(message: &WorkerMessage) {
    let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
    if let Ok(value) = serde_wasm_bindgen::to_value(message) {
        let envelope = js_sys::Object::new();
        if js_sys::Reflect::set(&envelope, &JsValue::from_str(MESSAGE_KEY), &value).is_ok() {
            drop(scope.post_message(&envelope));
        }
    }
}
