use bevy::{
    gltf::GltfAssetLabel,
    input::{
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton, MouseScrollUnit},
        ButtonInput,
    },
    prelude::*,
    scene::SceneRoot,
    window::{Window, WindowPlugin, WindowResolution},
};
use std::env;

const DEFAULT_MODEL_PATH: &str = "models/model.gltf";

#[derive(Resource, Clone)]
struct ViewerConfig {
    model_path: String,
}

#[derive(Component)]
struct ModelSpinner {
    speed: f32,
}

#[derive(Component)]
struct OrbitCamera {
    target: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
    orbit_sensitivity: f32,
    zoom_sensitivity: f32,
    min_radius: f32,
    max_radius: f32,
}

fn main() {
    let model_path = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_MODEL_PATH.to_string());

    App::new()
        .insert_resource(ViewerConfig { model_path })
        .insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "GLTF Preview".into(),
                resolution: WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (spin_model, orbit_camera))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<ViewerConfig>) {
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 8.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.5, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        OrbitCamera {
            target: Vec3::ZERO,
            radius: 6.0,
            yaw: 0.0,
            pitch: 0.25,
            orbit_sensitivity: 0.005,
            zoom_sensitivity: 0.8,
            min_radius: 1.5,
            max_radius: 30.0,
        },
    ));

    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(config.model_path.clone()));

    commands.spawn((
        SceneRoot(scene),
        Transform::default(),
        ModelSpinner {
            speed: std::f32::consts::TAU / 18.0,
        },
    ));
}

fn spin_model(time: Res<Time>, mut query: Query<(&mut Transform, &ModelSpinner)>) {
    for (mut transform, spinner) in &mut query {
        transform.rotate_y(spinner.speed * time.delta_secs());
    }
}

fn orbit_camera(
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let scroll_delta = normalized_scroll(&scroll);

    for (mut camera, mut transform) in &mut query {
        if buttons.pressed(MouseButton::Left) {
            camera.yaw -= motion.delta.x * camera.orbit_sensitivity;
            camera.pitch += motion.delta.y * camera.orbit_sensitivity;
            camera.pitch = camera.pitch.clamp(-1.45, 1.45);
        }

        if scroll_delta != 0.0 {
            camera.radius = (camera.radius - scroll_delta * camera.zoom_sensitivity)
                .clamp(camera.min_radius, camera.max_radius);
        }

        let x = camera.radius * camera.pitch.cos() * camera.yaw.sin();
        let y = camera.radius * camera.pitch.sin();
        let z = camera.radius * camera.pitch.cos() * camera.yaw.cos();

        transform.translation = camera.target + Vec3::new(x, y, z);
        transform.look_at(camera.target, Vec3::Y);
    }
}

fn normalized_scroll(scroll: &AccumulatedMouseScroll) -> f32 {
    match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR,
    }
}
