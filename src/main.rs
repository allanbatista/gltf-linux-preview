use bevy::{
    asset::{AssetPath, AssetPlugin, UnapprovedPathMode},
    camera::primitives::{Aabb, MeshAabb},
    gltf::GltfAssetLabel,
    input::{
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton, MouseScrollUnit},
        ButtonInput,
    },
    prelude::*,
    scene::SceneRoot,
    window::{PrimaryWindow, Window, WindowPlugin, WindowResolution},
};
use bevy_obj::ObjPlugin;
use std::env;
use std::path::{Path, PathBuf};

const DEFAULT_MODEL_PATH: &str = "models/model.gltf";
const MODEL_SPIN_SPEED: f32 = std::f32::consts::TAU / 18.0;

#[derive(Resource, Clone)]
struct ViewerConfig {
    model_path: PathBuf,
}

#[derive(Resource, Default)]
struct AnimationPlayback {
    has_animation: bool,
    paused: bool,
}

#[derive(Component)]
struct ModelSpinner {
    speed: f32,
}

#[derive(Component)]
struct ControlledAnimationPlayer;

#[derive(Component)]
struct AnimationToggleButton;

#[derive(Component)]
struct AnimationToggleLabel;

#[derive(Component)]
struct PendingCentering;

#[derive(Resource, Default)]
struct PendingCameraFit {
    radius: Option<f32>,
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
    let model_path = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_MODEL_PATH));
    let asset_root = determine_asset_root();

    App::new()
        .insert_resource(ViewerConfig { model_path })
        .insert_resource(AnimationPlayback::default())
        .insert_resource(PendingCameraFit::default())
        .insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.09)))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.display().to_string(),
                    unapproved_path_mode: UnapprovedPathMode::Deny,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "GLTF Preview".into(),
                        resolution: WindowResolution::new(1280, 720),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(ObjPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spin_model,
                orbit_camera,
                play_initial_animation,
                toggle_animation_playback,
                sync_animation_button,
            ),
        )
        .add_systems(
            PostUpdate,
            (
                center_pending_model.after(TransformSystems::Propagate),
                apply_pending_camera_fit.after(center_pending_model),
            ),
        )
        .run();
}

fn determine_asset_root() -> PathBuf {
    let cwd_assets = env::current_dir()
        .ok()
        .map(|dir| dir.join("assets"))
        .filter(|path| path.is_dir());
    if let Some(path) = cwd_assets {
        return path;
    }

    let exe_assets = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(|dir| dir.join("assets"))
        .filter(|path| path.is_dir());
    if let Some(path) = exe_assets {
        return path;
    }

    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("assets")
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

    let scene = load_model_scene(&asset_server, config.model_path.clone());

    commands.spawn((Transform::default(),)).with_children(move |parent| {
        parent.spawn((SceneRoot(scene), Transform::default(), PendingCentering));
    });

    commands
        .spawn((
            Button,
            Node {
                position_type: PositionType::Absolute,
                left: px(16),
                bottom: px(16),
                min_width: px(88),
                height: px(40),
                padding: UiRect::axes(px(16), px(8)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.14, 0.17, 0.92)),
            Visibility::Hidden,
            AnimationToggleButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Pause"),
                TextColor(Color::srgb(0.94, 0.95, 0.96)),
                AnimationToggleLabel,
            ));
        });
}

fn load_model_scene(asset_server: &AssetServer, model_path: PathBuf) -> Handle<Scene> {
    let scene_path = AssetPath::from_path_buf(model_path.clone());
    match model_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("obj") => asset_server.load_override(scene_path),
        _ => asset_server.load_override(GltfAssetLabel::Scene(0).from_asset(scene_path)),
    }
}

fn spin_model(time: Res<Time>, mut query: Query<(&mut Transform, &ModelSpinner)>) {
    for (mut transform, spinner) in &mut query {
        transform.rotate_y(spinner.speed * time.delta_secs());
    }
}

fn play_initial_animation(
    mut commands: Commands,
    config: Res<ViewerConfig>,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut playback: ResMut<AnimationPlayback>,
    mut query: Query<(Entity, &mut AnimationPlayer), Without<AnimationGraphHandle>>,
) {
    if query.is_empty() || is_obj_path(&config.model_path) {
        return;
    }

    let animation_path = GltfAssetLabel::Animation(0)
        .from_asset(AssetPath::from_path_buf(config.model_path.clone()));
    let animation = asset_server.load_override(animation_path);
    let (graph, animation_node) = AnimationGraph::from_clip(animation);
    let graph = animation_graphs.add(graph);

    for (entity, mut player) in &mut query {
        commands
            .entity(entity)
            .insert((AnimationGraphHandle(graph.clone()), ControlledAnimationPlayer));

        let active_animation = player.play(animation_node).repeat();
        if playback.paused {
            active_animation.pause();
        }
    }

    playback.has_animation = true;
}

fn toggle_animation_playback(
    mut playback: ResMut<AnimationPlayback>,
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<AnimationToggleButton>),
    >,
    mut players: Query<&mut AnimationPlayer, With<ControlledAnimationPlayer>>,
) {
    if !playback.has_animation {
        return;
    }

    for (interaction, mut color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                playback.paused = !playback.paused;
                for mut player in &mut players {
                    if playback.paused {
                        player.pause_all();
                    } else {
                        player.resume_all();
                    }
                }
                *color = BackgroundColor(Color::srgba(0.20, 0.23, 0.27, 0.96));
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgba(0.18, 0.20, 0.24, 0.96));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgba(0.12, 0.14, 0.17, 0.92));
            }
        }
    }
}

fn sync_animation_button(
    playback: Res<AnimationPlayback>,
    mut buttons: Query<&mut Visibility, With<AnimationToggleButton>>,
    mut labels: Query<&mut Text, With<AnimationToggleLabel>>,
) {
    if !playback.is_changed() {
        return;
    }

    let visibility = if playback.has_animation {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut button_visibility in &mut buttons {
        *button_visibility = visibility;
    }

    let label = if playback.paused { "Play" } else { "Pause" };
    for mut text in &mut labels {
        *text = Text::new(label);
    }
}

fn is_obj_path(path: &PathBuf) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("obj"))
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

        apply_orbit_transform(&camera, &mut transform);
    }
}

fn normalized_scroll(scroll: &AccumulatedMouseScroll) -> f32 {
    match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR,
    }
}

fn center_pending_model(
    mut commands: Commands,
    children: Query<&Children>,
    transforms: Query<&GlobalTransform>,
    mesh_query: Query<&Mesh3d>,
    meshes: Res<Assets<Mesh>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut pending_camera_fit: ResMut<PendingCameraFit>,
    mut query: Query<(Entity, &mut Transform, &ChildOf), With<PendingCentering>>,
) {
    let mut corners = Vec::new();
    let aspect_ratio = windows
        .iter()
        .next()
        .map(|window| {
            if window.height() > 0.0 {
                window.width() / window.height()
            } else {
                16.0 / 9.0
            }
        })
        .unwrap_or(16.0 / 9.0);

    for (entity, mut transform, child_of) in &mut query {
        corners.clear();

        accumulate_mesh_corners(entity, &children, &transforms, &mesh_query, &meshes, &mut corners);

        if let Some(aabb) = Aabb::enclosing(corners.iter().copied()) {
            transform.translation = -Vec3::from(aabb.center);
            pending_camera_fit.radius = Some(fit_camera_radius(&aabb, aspect_ratio));

            commands
                .entity(child_of.parent())
                .insert(ModelSpinner {
                    speed: MODEL_SPIN_SPEED,
                });
            commands.entity(entity).remove::<PendingCentering>();
        }
    }
}

fn apply_pending_camera_fit(
    mut pending_camera_fit: ResMut<PendingCameraFit>,
    mut query: Query<(&mut OrbitCamera, &mut Transform), With<Camera3d>>,
) {
    let Some(radius) = pending_camera_fit.radius.take() else {
        return;
    };

    if let Some((mut camera, mut transform)) = query.iter_mut().next() {
        camera.target = Vec3::ZERO;
        camera.radius = radius;
        camera.min_radius = radius * 0.85;
        camera.max_radius = radius * 20.0;
        apply_orbit_transform(&camera, &mut transform);
    }
}

fn fit_camera_radius(aabb: &Aabb, aspect_ratio: f32) -> f32 {
    let sphere_radius = aabb.half_extents.length().max(0.5);
    let vertical_half_fov = std::f32::consts::FRAC_PI_8;
    let horizontal_half_fov = (vertical_half_fov.tan() * aspect_ratio.max(0.1)).atan();
    let half_fov = vertical_half_fov.min(horizontal_half_fov);

    sphere_radius / half_fov.sin() * 1.25
}

fn apply_orbit_transform(camera: &OrbitCamera, transform: &mut Transform) {
    let x = camera.radius * camera.pitch.cos() * camera.yaw.sin();
    let y = camera.radius * camera.pitch.sin();
    let z = camera.radius * camera.pitch.cos() * camera.yaw.cos();

    transform.translation = camera.target + Vec3::new(x, y, z);
    transform.look_at(camera.target, Vec3::Y);
}

fn accumulate_mesh_corners(
    entity: Entity,
    child_query: &Query<&Children>,
    transforms: &Query<&GlobalTransform>,
    mesh_query: &Query<&Mesh3d>,
    meshes: &Assets<Mesh>,
    corners: &mut Vec<Vec3>,
) {
    if let Ok(mesh) = mesh_query.get(entity) {
        if let Some(mesh) = meshes.get(&mesh.0) {
            if let Some(aabb) = mesh.compute_aabb() {
                if let Ok(transform) = transforms.get(entity) {
                    let min: Vec3 = aabb.min().into();
                    let max: Vec3 = aabb.max().into();
                    let affine = transform.affine();
                    corners.extend([
                        affine.transform_point3(Vec3::new(min.x, min.y, min.z)),
                        affine.transform_point3(Vec3::new(min.x, min.y, max.z)),
                        affine.transform_point3(Vec3::new(min.x, max.y, min.z)),
                        affine.transform_point3(Vec3::new(min.x, max.y, max.z)),
                        affine.transform_point3(Vec3::new(max.x, min.y, min.z)),
                        affine.transform_point3(Vec3::new(max.x, min.y, max.z)),
                        affine.transform_point3(Vec3::new(max.x, max.y, min.z)),
                        affine.transform_point3(Vec3::new(max.x, max.y, max.z)),
                    ]);
                }
            }
        }
    }

    let Ok(entity_children) = child_query.get(entity) else {
        return;
    };

    for child in entity_children.iter() {
        accumulate_mesh_corners(
            child,
            child_query,
            transforms,
            mesh_query,
            meshes,
            corners,
        );
    }
}
