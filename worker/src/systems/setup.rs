use nightshade::ecs::loading::load_texture_pack_from_image_bytes;
use nightshade::prelude::{
    Atmosphere, Light, LightType, Projection, Vec3, load_procedural_textures,
    mark_local_transform_dirty, spawn_sun,
};
use nightshade::render::wgpu::texture_cache::{SamplerSettings, TextureUsage};
use nightshade_api::prelude::*;

pub const FOCUS: Vec3 = Vec3::new(0.0, 1.6, 0.0);
pub const RADIUS: f32 = 11.0;

/// The built-in prototype textures, embedded from the engine asset pack and
/// registered by name so the scripts can `set_texture` them onto the geometry.
const PROTOTYPE_TEXTURES: &[(&str, &[u8])] = &[
    (
        "proto_dark",
        include_bytes!("../../../assets/textures/prototype/dark/texture_06.png") as &[u8],
    ),
    (
        "proto_light",
        include_bytes!("../../../assets/textures/prototype/light/texture_01.png") as &[u8],
    ),
    (
        "proto_orange",
        include_bytes!("../../../assets/textures/prototype/orange/texture_01.png") as &[u8],
    ),
    (
        "proto_green",
        include_bytes!("../../../assets/textures/prototype/green/texture_01.png") as &[u8],
    ),
    (
        "proto_purple",
        include_bytes!("../../../assets/textures/prototype/purple/texture_01.png") as &[u8],
    ),
];

/// Builds the empty stage the snippets assemble onto: prototype textures, a
/// shadow-casting sun, and a compact orbit camera framed so the scene sits in the
/// near shadow cascade (crisp on the smaller wasm cascade atlas). The floor,
/// pillars, spheres, lights, and centerpiece all arrive later as scripts run.
pub fn initialize(world: &mut World) {
    if let Some((width, height)) = world.resources.window.cached_viewport_size {
        world.resources.window.active_viewport_rect =
            Some(nightshade::ecs::window::resources::ViewportRect {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            });
    }

    world.resources.physics.enabled = true;

    load_procedural_textures(world);
    load_texture_pack_from_image_bytes(
        world,
        PROTOTYPE_TEXTURES,
        TextureUsage::Color,
        SamplerSettings::DEFAULT,
    );
    initialize_draw_pools(world);

    spawn_shadow_sun(world);
    orbit_camera(world, FOCUS, RADIUS);
    tighten_shadow_cascades(world);
    apply_stage(world);
}

/// The directional shadow cascade split distances scale with the camera near
/// plane. Pulling the near plane in shrinks the cascades so this compact scene
/// fills the dense near cascade, which keeps shadows crisp on the smaller wasm
/// cascade atlas.
fn tighten_shadow_cascades(world: &mut World) {
    if let Some(camera) = world.resources.active_camera
        && let Some(component) = world.core.get_camera_mut(camera)
        && let Projection::Perspective(perspective) = &mut component.projection
    {
        perspective.z_near = 0.006;
    }
}

/// A directional sun angled across the scene that casts cascaded shadows.
fn spawn_shadow_sun(world: &mut World) {
    let sun = spawn_sun(world);
    if let Some(transform) = world.core.get_local_transform_mut(sun) {
        let pitch = nalgebra_glm::quat_angle_axis(-0.85, &Vec3::x_axis());
        let yaw = nalgebra_glm::quat_angle_axis(0.6, &Vec3::y_axis());
        transform.rotation = yaw * pitch;
    }
    mark_local_transform_dirty(world, sun);
    world.core.set_light(
        sun,
        Light {
            light_type: LightType::Directional,
            color: Vec3::new(1.0, 0.96, 0.88),
            intensity: 2.8,
            cast_shadows: true,
            shadow_bias: 0.0016,
            ..Default::default()
        },
    );
}

/// The plain stage state shared by setup and reset: daylight sky, soft ambient,
/// bloom for the emissive light markers, the reference grid, and the camera
/// framing. Re-applied on reset so the scene returns to this look.
pub fn apply_stage(world: &mut World) {
    world.resources.render_settings.atmosphere = Atmosphere::Sky;
    set_ambient(world, [0.18, 0.2, 0.26, 1.0]);
    set_bloom(world, true);
    show_grid(world, true);
    set_orbit_view(world, FOCUS, RADIUS, 0.7, 0.36);
}
