use bevy::{
    asset::{AssetId, AssetPath, AssetPlugin, UnapprovedPathMode},
    camera::{
        primitives::{Aabb, MeshAabb},
        RenderTarget,
    },
    gltf::{Gltf, GltfAssetLabel},
    input::{
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton, MouseScrollUnit},
        ButtonInput,
    },
    pbr::wireframe::{Wireframe, WireframeColor, WireframePlugin},
    prelude::*,
    render::{
        render_resource::{PrimitiveTopology, TextureFormat, WgpuFeatures},
        renderer::RenderDevice,
    },
    window::{PrimaryWindow, Window, WindowPlugin, WindowResolution},
};
use bevy_egui::{
    egui, input::EguiWantsInput, EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    EguiTextureHandle, EguiUserTextures,
};
use bevy_obj::ObjPlugin;
use std::{
    collections::{HashMap, HashSet},
    env,
    path::{Path, PathBuf},
};

const DEFAULT_MODEL_PATH: &str = "models/model.gltf";
const MODEL_SPIN_SPEED: f32 = std::f32::consts::TAU / 18.0;
const THUMBNAIL_SIZE: u32 = 160;

#[derive(Resource, Clone)]
struct ViewerConfig {
    model_path: PathBuf,
}

#[derive(Resource, Default)]
struct ModelAssets {
    gltf: Option<Handle<Gltf>>,
}

#[derive(Resource, Default)]
struct ModelStats {
    polygons: usize,
    faces: usize,
    vram_bytes: usize,
    available: bool,
}

#[derive(Resource)]
struct AnimationPlayback {
    initialized: bool,
    has_animation: bool,
    paused: bool,
    looping: bool,
    selected: usize,
    names: Vec<String>,
    nodes: Vec<AnimationNodeIndex>,
}

impl Default for AnimationPlayback {
    fn default() -> Self {
        Self {
            initialized: false,
            has_animation: false,
            paused: false,
            looping: true,
            selected: 0,
            names: Vec::new(),
            nodes: Vec::new(),
        }
    }
}

#[derive(Resource)]
struct AutoRotation(bool);

impl Default for AutoRotation {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Resource, Default)]
struct ThumbnailState {
    requested: bool,
    camera: Option<Entity>,
    frames_remaining: u8,
}

#[derive(Resource)]
struct ThumbnailTarget(Handle<Image>);

#[derive(Resource, Default)]
struct ViewMaterialCache {
    textured: HashMap<AssetId<StandardMaterial>, Handle<StandardMaterial>>,
    untextured: Option<Handle<StandardMaterial>>,
}

#[derive(Resource, Default)]
struct FlatMeshCache(HashMap<AssetId<Mesh>, Handle<Mesh>>);

#[derive(Resource, Default)]
struct ViewSettings {
    mode: ViewMode,
    wireframe: bool,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum ViewMode {
    #[default]
    Rendered,
    Textured,
    Smooth,
    Solid,
}

impl ViewMode {
    fn allows_wireframe(self) -> bool {
        !matches!(self, Self::Rendered)
    }
}

#[derive(Component)]
struct ModelSpinner {
    speed: f32,
}

#[derive(Component)]
struct ControlledAnimationPlayer;

#[derive(Component)]
struct ModelMesh {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Component)]
struct PendingCentering;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct ThumbnailCamera;

#[derive(Component)]
struct CameraLight;

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
        .insert_resource(ModelAssets::default())
        .insert_resource(ModelStats::default())
        .insert_resource(AnimationPlayback::default())
        .insert_resource(AutoRotation::default())
        .insert_resource(ThumbnailState::default())
        .insert_resource(ViewMaterialCache::default())
        .insert_resource(FlatMeshCache::default())
        .insert_resource(ViewSettings::default())
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
        .add_plugins((ObjPlugin, WireframePlugin::default(), EguiPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spin_model,
                orbit_camera,
                capture_model_meshes,
                setup_animation_graph,
                apply_view_settings.after(capture_model_meshes),
                stop_thumbnail_camera,
            ),
        )
        .add_systems(EguiPrimaryContextPass, draw_ui)
        .add_systems(
            PostUpdate,
            (
                center_pending_model.after(TransformSystems::Propagate),
                apply_pending_camera_fit.after(center_pending_model),
                sync_camera_light.after(apply_pending_camera_fit),
                spawn_thumbnail_camera.after(sync_camera_light),
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

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<ViewerConfig>,
    mut images: ResMut<Assets<Image>>,
    mut egui_textures: ResMut<EguiUserTextures>,
) {
    let thumbnail = images.add(Image::new_target_texture(
        THUMBNAIL_SIZE,
        THUMBNAIL_SIZE,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    ));
    egui_textures.add_image(EguiTextureHandle::Strong(thumbnail.clone()));
    commands.insert_resource(ThumbnailTarget(thumbnail));

    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 1.5, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraLight,
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
        MainCamera,
    ));

    let (scene, gltf) = load_model_scene(&asset_server, &config.model_path);
    commands.insert_resource(ModelAssets { gltf });
    commands
        .spawn((Transform::default(), Visibility::default()))
        .with_children(move |parent| {
            parent.spawn((
                WorldAssetRoot(scene),
                Transform::default(),
                PendingCentering,
            ));
        });
}

fn load_model_scene(
    asset_server: &AssetServer,
    model_path: &Path,
) -> (Handle<WorldAsset>, Option<Handle<Gltf>>) {
    let scene_path = AssetPath::from_path_buf(model_path.to_path_buf());
    if is_obj_path(model_path) {
        return (
            asset_server
                .load_builder()
                .override_unapproved()
                .load(scene_path),
            None,
        );
    }

    let gltf = asset_server
        .load_builder()
        .override_unapproved()
        .load(scene_path.clone());
    let scene = asset_server
        .load_builder()
        .override_unapproved()
        .load(GltfAssetLabel::Scene(0).from_asset(scene_path));
    (scene, Some(gltf))
}

fn spin_model(
    time: Res<Time>,
    auto_rotation: Res<AutoRotation>,
    mut query: Query<(&mut Transform, &ModelSpinner)>,
) {
    if !auto_rotation.0 {
        return;
    }

    for (mut transform, spinner) in &mut query {
        transform.rotate_y(spinner.speed * time.delta_secs());
    }
}

fn setup_animation_graph(
    mut commands: Commands,
    model_assets: Res<ModelAssets>,
    gltfs: Res<Assets<Gltf>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut playback: ResMut<AnimationPlayback>,
    mut players: Query<(Entity, &mut AnimationPlayer), Without<ControlledAnimationPlayer>>,
) {
    if playback.initialized {
        return;
    }

    let Some(gltf_handle) = model_assets.gltf.as_ref() else {
        return;
    };
    let Some(gltf) = gltfs.get(gltf_handle) else {
        return;
    };

    if gltf.animations.is_empty() {
        playback.initialized = true;
        return;
    }
    if players.is_empty() {
        return;
    }

    let (graph, nodes) = AnimationGraph::from_clips(gltf.animations.iter().cloned());
    let graph = animation_graphs.add(graph);
    playback.names = gltf
        .animations
        .iter()
        .enumerate()
        .map(|(index, animation)| {
            gltf.named_animations
                .iter()
                .find_map(|(name, handle)| (handle == animation).then(|| name.to_string()))
                .unwrap_or_else(|| format!("Animacao {}", index + 1))
        })
        .collect();
    playback.nodes = nodes;
    playback.selected = 0;
    playback.has_animation = true;
    playback.initialized = true;

    for (entity, mut player) in &mut players {
        commands.entity(entity).insert((
            AnimationGraphHandle(graph.clone()),
            ControlledAnimationPlayer,
        ));
        start_selected_animation(&mut player, &playback);
    }
}

fn start_selected_animation(player: &mut AnimationPlayer, playback: &AnimationPlayback) {
    let Some(&node) = playback.nodes.get(playback.selected) else {
        return;
    };

    player.stop_all();
    let animation = player.start(node);
    if playback.looping {
        animation.repeat();
    }
    if playback.paused {
        animation.pause();
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_ui(
    mut contexts: EguiContexts,
    thumbnail: Option<Res<ThumbnailTarget>>,
    stats: Res<ModelStats>,
    mut auto_rotation: ResMut<AutoRotation>,
    mut playback: ResMut<AnimationPlayback>,
    mut view_settings: ResMut<ViewSettings>,
    render_device: Option<Res<RenderDevice>>,
    mut players: Query<&mut AnimationPlayer, With<ControlledAnimationPlayer>>,
) -> Result {
    let thumbnail_id = thumbnail
        .as_ref()
        .and_then(|thumbnail| contexts.image_id(&thumbnail.0));
    let context = contexts.ctx_mut()?;

    egui::Window::new("Modelo")
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(16.0, 16.0))
        .resizable(false)
        .collapsible(false)
        .show(context, |ui| {
            if let Some(texture_id) = thumbnail_id {
                ui.image(egui::load::SizedTexture::new(
                    texture_id,
                    egui::vec2(THUMBNAIL_SIZE as f32, THUMBNAIL_SIZE as f32),
                ));
            }
            if stats.available {
                ui.label(format!("Poligonos: {}", stats.polygons));
                ui.label(format!("Faces: {}", stats.faces));
                ui.label(format!("VRAM estimada: {}", format_bytes(stats.vram_bytes)));
            } else {
                ui.label("Estatisticas carregando...");
            }
            ui.separator();
            let label = if auto_rotation.0 {
                "Rotacao: ligada"
            } else {
                "Rotacao: desligada"
            };
            if ui.button(label).clicked() {
                auto_rotation.0 = !auto_rotation.0;
            }
        });

    let mut selected_animation = None;
    let mut toggle_pause = false;
    let mut toggle_loop = false;
    egui::Window::new("Animacoes")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
        .resizable(false)
        .collapsible(false)
        .show(context, |ui| {
            if !playback.has_animation {
                ui.label("Sem animacoes");
                return;
            }

            let selected_name = playback
                .names
                .get(playback.selected)
                .map(String::as_str)
                .unwrap_or("Animacao");
            ui.label(format!("Selecionada: {selected_name}"));
            ui.horizontal(|ui| {
                let play_label = if playback.paused {
                    "Reproduzir"
                } else {
                    "Pausar"
                };
                if ui.button(play_label).clicked() {
                    toggle_pause = true;
                }
                if ui
                    .add(egui::Button::selectable(playback.looping, "Loop"))
                    .clicked()
                {
                    toggle_loop = true;
                }
            });
            ui.separator();
            for index in 0..playback.names.len() {
                let name = playback.names[index].as_str();
                if ui
                    .add(egui::Button::selectable(index == playback.selected, name))
                    .clicked()
                {
                    selected_animation = Some(index);
                }
            }
        });

    let mut mode = view_settings.mode;
    let mut wireframe = view_settings.wireframe;
    egui::Area::new(egui::Id::new("view-mode-controls"))
        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -16.0))
        .show(context, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::selectable(
                        mode == ViewMode::Rendered,
                        "Renderizado",
                    ))
                    .clicked()
                {
                    mode = ViewMode::Rendered;
                }
                if ui
                    .add(egui::Button::selectable(
                        mode == ViewMode::Textured,
                        "Texturizado",
                    ))
                    .clicked()
                {
                    mode = ViewMode::Textured;
                }
                if ui
                    .add(egui::Button::selectable(mode == ViewMode::Smooth, "Suave"))
                    .clicked()
                {
                    mode = ViewMode::Smooth;
                }
                if ui
                    .add(egui::Button::selectable(mode == ViewMode::Solid, "Solido"))
                    .clicked()
                {
                    mode = ViewMode::Solid;
                }
                let wireframe_available = wireframe_supported(render_device.as_deref());
                if !mode.allows_wireframe() || !wireframe_available {
                    wireframe = false;
                }
                let response = ui.add_enabled(
                    mode.allows_wireframe() && wireframe_available,
                    egui::Button::selectable(wireframe, "Wireframe"),
                );
                let clicked = response.clicked();
                if !wireframe_available {
                    response.on_disabled_hover_text("Wireframe indisponivel nesta GPU");
                }
                if clicked {
                    wireframe = !wireframe;
                }
            });
        });

    if let Some(selected) = selected_animation {
        playback.selected = selected;
    }
    if toggle_loop {
        playback.looping = !playback.looping;
    }
    if selected_animation.is_some() || toggle_loop {
        for mut player in &mut players {
            start_selected_animation(&mut player, &playback);
        }
    }
    if toggle_pause {
        playback.paused = !playback.paused;
        for mut player in &mut players {
            if playback.paused {
                player.pause_all();
            } else {
                player.resume_all();
            }
        }
    }
    if mode != view_settings.mode {
        wireframe = false;
    }
    if mode != view_settings.mode || wireframe != view_settings.wireframe {
        view_settings.mode = mode;
        view_settings.wireframe = wireframe;
    }

    Ok(())
}

fn capture_model_meshes(
    mut commands: Commands,
    query: Query<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>), Without<ModelMesh>>,
) {
    for (entity, mesh, material) in &query {
        commands.entity(entity).insert(ModelMesh {
            mesh: mesh.0.clone(),
            material: material.0.clone(),
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_view_settings(
    mut commands: Commands,
    settings: Res<ViewSettings>,
    render_device: Option<Res<RenderDevice>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material_cache: ResMut<ViewMaterialCache>,
    mut flat_meshes: ResMut<FlatMeshCache>,
    mut query: Query<(
        Entity,
        Ref<ModelMesh>,
        &mut Mesh3d,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let settings_changed = settings.is_changed();
    let wireframe_available = wireframe_supported(render_device.as_deref());
    for (entity, original, mut mesh, mut material) in &mut query {
        if !settings_changed && !original.is_added() {
            continue;
        }

        mesh.0 = if settings.mode == ViewMode::Solid {
            flat_mesh_handle(&original.mesh, &mut meshes, &mut flat_meshes)
        } else {
            original.mesh.clone()
        };
        material.0 = match settings.mode {
            ViewMode::Rendered => original.material.clone(),
            ViewMode::Textured => {
                textured_material_handle(&original.material, &mut materials, &mut material_cache)
            }
            ViewMode::Smooth | ViewMode::Solid => {
                untextured_material_handle(&mut materials, &mut material_cache)
            }
        };

        if settings.wireframe && settings.mode.allows_wireframe() && wireframe_available {
            commands.entity(entity).insert((
                Wireframe,
                WireframeColor {
                    color: Color::srgb(0.96, 0.97, 0.99),
                },
            ));
        } else {
            commands.entity(entity).remove::<Wireframe>();
            commands.entity(entity).remove::<WireframeColor>();
        }
    }
}

fn wireframe_supported(render_device: Option<&RenderDevice>) -> bool {
    render_device.is_some_and(|device| {
        device
            .features()
            .contains(WgpuFeatures::POLYGON_MODE_LINE | WgpuFeatures::IMMEDIATES)
    })
}

fn textured_material_handle(
    original: &Handle<StandardMaterial>,
    materials: &mut Assets<StandardMaterial>,
    cache: &mut ViewMaterialCache,
) -> Handle<StandardMaterial> {
    if let Some(material) = cache.textured.get(&original.id()) {
        return material.clone();
    }

    let mut material = materials.get(original).cloned().unwrap_or_default();
    material.unlit = true;
    let handle = materials.add(material);
    cache.textured.insert(original.id(), handle.clone());
    handle
}

fn untextured_material_handle(
    materials: &mut Assets<StandardMaterial>,
    cache: &mut ViewMaterialCache,
) -> Handle<StandardMaterial> {
    if let Some(material) = cache.untextured.as_ref() {
        return material.clone();
    }

    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.74, 0.78),
        metallic: 0.0,
        perceptual_roughness: 0.8,
        ..default()
    });
    cache.untextured = Some(material.clone());
    material
}

fn flat_mesh_handle(
    original: &Handle<Mesh>,
    meshes: &mut Assets<Mesh>,
    cache: &mut FlatMeshCache,
) -> Handle<Mesh> {
    if let Some(mesh) = cache.0.get(&original.id()) {
        return mesh.clone();
    }
    let Some(source) = meshes.get(original).cloned() else {
        return original.clone();
    };
    if source.primitive_topology() != PrimitiveTopology::TriangleList {
        return original.clone();
    }
    let Ok(mut flat) = source.try_with_duplicated_vertices() else {
        return original.clone();
    };
    if flat.try_compute_flat_normals().is_err() {
        return original.clone();
    }

    let handle = meshes.add(flat);
    cache.0.insert(original.id(), handle.clone());
    handle
}

fn orbit_camera(
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    buttons: Res<ButtonInput<MouseButton>>,
    egui_input: Res<EguiWantsInput>,
    mut query: Query<(&mut OrbitCamera, &mut Transform), With<MainCamera>>,
) {
    if egui_input.wants_any_pointer_input() {
        return;
    }

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

#[allow(clippy::too_many_arguments)]
fn center_pending_model(
    mut commands: Commands,
    children: Query<&Children>,
    transforms: Query<&GlobalTransform>,
    mesh_query: Query<&Mesh3d>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    model_meshes: Query<&ModelMesh>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    images: Res<Assets<Image>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut pending_camera_fit: ResMut<PendingCameraFit>,
    mut stats: ResMut<ModelStats>,
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
        accumulate_mesh_corners(
            entity,
            &children,
            &transforms,
            &mesh_query,
            &meshes,
            &mut corners,
        );

        if let Some(aabb) = Aabb::enclosing(corners.iter().copied()) {
            transform.translation = -Vec3::from(aabb.center);
            pending_camera_fit.radius = Some(fit_camera_radius(&aabb, aspect_ratio));

            let mut model_stats = ModelStats::default();
            let mut mesh_ids = HashSet::new();
            let mut material_ids = HashSet::new();
            let mut image_ids = HashSet::new();
            accumulate_model_stats(
                entity,
                &children,
                &mesh_query,
                &material_query,
                &model_meshes,
                &meshes,
                &materials,
                &images,
                &mut model_stats,
                &mut mesh_ids,
                &mut material_ids,
                &mut image_ids,
            );
            model_stats.available = true;
            *stats = model_stats;

            commands.entity(child_of.parent()).insert(ModelSpinner {
                speed: MODEL_SPIN_SPEED,
            });
            commands.entity(entity).remove::<PendingCentering>();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn accumulate_model_stats(
    entity: Entity,
    child_query: &Query<&Children>,
    mesh_query: &Query<&Mesh3d>,
    material_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    model_meshes: &Query<&ModelMesh>,
    meshes: &Assets<Mesh>,
    materials: &Assets<StandardMaterial>,
    images: &Assets<Image>,
    stats: &mut ModelStats,
    mesh_ids: &mut HashSet<AssetId<Mesh>>,
    material_ids: &mut HashSet<AssetId<StandardMaterial>>,
    image_ids: &mut HashSet<AssetId<Image>>,
) {
    if let Ok(mesh_component) = mesh_query.get(entity) {
        let (mesh_handle, material_handle) = if let Ok(model_mesh) = model_meshes.get(entity) {
            (model_mesh.mesh.clone(), Some(model_mesh.material.clone()))
        } else {
            (
                mesh_component.0.clone(),
                material_query
                    .get(entity)
                    .ok()
                    .map(|material| material.0.clone()),
            )
        };

        if let Some(mesh) = meshes.get(&mesh_handle) {
            let element_count = mesh
                .indices()
                .map_or_else(|| mesh.count_vertices(), |indices| indices.len());
            stats.polygons += triangle_count(mesh.primitive_topology(), element_count);
            stats.faces += triangle_count(mesh.primitive_topology(), element_count);

            if mesh_ids.insert(mesh_handle.id()) {
                stats.vram_bytes += mesh.get_vertex_buffer_size();
                stats.vram_bytes += mesh.get_index_buffer_bytes().map_or(0, <[u8]>::len);
            }
        }

        if let Some(material_handle) = material_handle {
            if material_ids.insert(material_handle.id()) {
                if let Some(material) = materials.get(&material_handle) {
                    stats.vram_bytes += material_texture_bytes(material, images, image_ids);
                }
            }
        }
    }

    let Ok(entity_children) = child_query.get(entity) else {
        return;
    };
    for child in entity_children.iter() {
        accumulate_model_stats(
            child,
            child_query,
            mesh_query,
            material_query,
            model_meshes,
            meshes,
            materials,
            images,
            stats,
            mesh_ids,
            material_ids,
            image_ids,
        );
    }
}

fn triangle_count(topology: PrimitiveTopology, element_count: usize) -> usize {
    match topology {
        PrimitiveTopology::TriangleList => element_count / 3,
        PrimitiveTopology::TriangleStrip => element_count.saturating_sub(2),
        _ => 0,
    }
}

fn material_texture_bytes(
    material: &StandardMaterial,
    images: &Assets<Image>,
    image_ids: &mut HashSet<AssetId<Image>>,
) -> usize {
    [
        material.base_color_texture.as_ref(),
        material.emissive_texture.as_ref(),
        material.metallic_roughness_texture.as_ref(),
        material.normal_map_texture.as_ref(),
        material.occlusion_texture.as_ref(),
    ]
    .into_iter()
    .flatten()
    .filter_map(|texture| {
        image_ids
            .insert(texture.id())
            .then(|| images.get(texture))
            .flatten()
            .and_then(|image| image.data.as_ref())
            .map(Vec::len)
    })
    .sum()
}

fn apply_pending_camera_fit(
    mut pending_camera_fit: ResMut<PendingCameraFit>,
    mut thumbnail_state: ResMut<ThumbnailState>,
    mut query: Query<(&mut OrbitCamera, &mut Transform), With<MainCamera>>,
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
        thumbnail_state.requested = true;
    }
}

fn sync_camera_light(
    camera: Query<&Transform, (With<MainCamera>, Without<CameraLight>)>,
    mut lights: Query<&mut Transform, (With<CameraLight>, Without<MainCamera>)>,
) {
    let Some(camera_transform) = camera.iter().next() else {
        return;
    };
    for mut light_transform in &mut lights {
        *light_transform = *camera_transform;
    }
}

fn spawn_thumbnail_camera(
    mut commands: Commands,
    target: Option<Res<ThumbnailTarget>>,
    mut state: ResMut<ThumbnailState>,
    camera: Query<&Transform, With<MainCamera>>,
) {
    if !state.requested || state.camera.is_some() {
        return;
    }
    let Some(target) = target else {
        return;
    };
    let Some(transform) = camera.iter().next() else {
        return;
    };

    let thumbnail_camera = commands
        .spawn((
            Camera3d::default(),
            Camera {
                order: -1,
                ..default()
            },
            RenderTarget::Image(target.0.clone().into()),
            *transform,
            ThumbnailCamera,
        ))
        .id();
    state.requested = false;
    state.camera = Some(thumbnail_camera);
    state.frames_remaining = 2;
}

fn stop_thumbnail_camera(
    mut state: ResMut<ThumbnailState>,
    mut cameras: Query<&mut Camera, With<ThumbnailCamera>>,
) {
    let Some(entity) = state.camera else {
        return;
    };
    if state.frames_remaining > 1 {
        state.frames_remaining -= 1;
        return;
    }
    state.camera = None;
    state.frames_remaining = 0;
    if let Ok(mut camera) = cameras.get_mut(entity) {
        camera.is_active = false;
    }
}

fn format_bytes(bytes: usize) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn is_obj_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("obj"))
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
        accumulate_mesh_corners(child, child_query, transforms, mesh_query, meshes, corners);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_triangle_topologies() {
        assert_eq!(triangle_count(PrimitiveTopology::TriangleList, 6), 2);
        assert_eq!(triangle_count(PrimitiveTopology::TriangleStrip, 4), 2);
        assert_eq!(triangle_count(PrimitiveTopology::TriangleList, 0), 0);
    }

    #[test]
    fn formats_vram_values() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1_572_864), "1.5 MiB");
    }

    #[test]
    fn defaults_to_looped_animation_and_rendered_mode() {
        let playback = AnimationPlayback::default();
        assert!(playback.looping);
        assert!(!playback.paused);
        assert_eq!(playback.selected, 0);
        assert!(!ViewMode::Rendered.allows_wireframe());
        assert!(ViewMode::Textured.allows_wireframe());
    }
}
