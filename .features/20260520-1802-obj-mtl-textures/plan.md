# Plan: OBJ/MTL support and external glTF textures

Status: READY_FOR_EXEC

## Summary

Add practical local OBJ preview support without changing the viewer UX. Keep glTF/GLB on Bevy's glTF scene loader, add `bevy_obj` for `.obj` scenes, enable PNG image loading for common sidecar textures, and commit small validation fixtures.

## Implementation

- Runtime loader: add `bevy_obj = "0.18.2"`, register `ObjPlugin`, and dispatch `.obj` paths to `AssetServer::load_override::<Scene>` while glTF/GLB continue through `GltfAssetLabel::Scene(0)`.
- Texture support: add Bevy `png` feature so glTF external PNG images and OBJ/MTL `map_Kd` PNG textures can load. Keep existing `jpeg` and `tga`.
- Fixtures: add Khronos `BoxTextured` glTF fixture with external `.bin` and `.png`; add textured OBJ fixture with `cube-tex.obj`, `cube.mtl`, and `texture.png`.
- Public surface: update README, `assets/models/README.md`, and install scripts so `.obj` is documented and desktop MIME includes `model/obj`.

## Acceptance Mapping

- AC-01, AC-02, AC-03, AC-08, AC-10, AC-13: covered by loader dispatch, `bevy_obj`, PNG feature, and validation fixtures.
- AC-12: covered by README and installer desktop entry updates.
- AC-04, AC-05, AC-06, AC-07, AC-09, AC-11: delegated to Bevy/bevy_obj behavior where possible; remaining edge cases require manual/e2e evidence if marked complete in a release gate.
- AC-14: covered by `cargo check` plus fixture presence; visual e2e remains recommended for release.

## Validation

- `rtk cargo check`
- `rtk rg -n '"uri":|mtllib|usemtl|map_Kd' assets/models/fixtures`
- Manual visual checks:
  - `rtk cargo run -- models/fixtures/box-textured/BoxTextured.gltf`
  - `rtk cargo run -- models/fixtures/cube-obj/cube-tex.obj`

## Assumptions

- OBJ/MTL support is intentionally scoped to `bevy_obj`'s scene loader and material support.
- Network asset fetching stays unsupported.
- The app remains Linux-focused and keeps the existing one-argument CLI contract.
