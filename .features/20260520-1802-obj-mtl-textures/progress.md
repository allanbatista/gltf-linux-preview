# Progress: OBJ/MTL support and external glTF textures

Status: IMPLEMENTED_PENDING_VISUAL_E2E

## Tasks

- [x] Spec created by `feature-spec`.
  - Files: `.features/20260520-1802-obj-mtl-textures/spec.md`
  - Evidence: `Status: READY_FOR_PLAN`
- [x] Plan created.
  - Files: `.features/20260520-1802-obj-mtl-textures/plan.md`
  - Evidence: `Status: READY_FOR_EXEC`
- [x] Runtime loader implemented.
  - Files: `Cargo.toml`, `Cargo.lock`, `src/main.rs`
  - Evidence: `rtk cargo check` passed
- [x] Fixtures added.
  - Files: `assets/models/fixtures/box-textured/*`, `assets/models/fixtures/cube-obj/*`
  - Evidence: sidecar glTF URIs and OBJ/MTL references present
- [x] Documentation and installer metadata updated.
  - Files: `README.md`, `assets/models/README.md`, `build-and-install.sh`, `install.sh`
  - Evidence: `.obj` usage and `model/obj` MIME documented
- [ ] Visual screenshot validation.
  - Owner: e2e-validator or manual tester
  - Required evidence: screenshots for both fixtures
  - Blocker: not captured in this implementation pass

## Actual Files

- `Cargo.toml`
- `Cargo.lock`
- `src/main.rs`
- `README.md`
- `assets/models/README.md`
- `build-and-install.sh`
- `install.sh`
- `.features/20260520-1802-obj-mtl-textures/spec.md`
- `.features/20260520-1802-obj-mtl-textures/plan.md`
- `.features/20260520-1802-obj-mtl-textures/progress.md`
- `assets/models/fixtures/box-textured/BoxTextured.gltf`
- `assets/models/fixtures/box-textured/BoxTextured0.bin`
- `assets/models/fixtures/box-textured/CesiumLogoFlat.png`
- `assets/models/fixtures/box-textured/LICENSE.md`
- `assets/models/fixtures/box-textured/SOURCE.md`
- `assets/models/fixtures/cube-obj/cube-tex.obj`
- `assets/models/fixtures/cube-obj/cube.mtl`
- `assets/models/fixtures/cube-obj/texture.png`
- `assets/models/fixtures/cube-obj/SOURCE.md`

## Evidence

- `rtk cargo check`: passed.
- `rtk cargo check --locked`: passed.
- `rtk timeout 10s cargo run -- models/fixtures/box-textured/BoxTextured.gltf`: opened Bevy window; no missing asset errors before intentional timeout.
- `rtk timeout 10s cargo run -- models/fixtures/cube-obj/cube-tex.obj`: opened Bevy window; no missing asset errors before intentional timeout.
- `assets/models/fixtures/box-textured/BoxTextured.gltf`: references `CesiumLogoFlat.png` and `BoxTextured0.bin`.
- `assets/models/fixtures/cube-obj/cube-tex.obj`: references `mtllib cube.mtl` and `usemtl texture`.
- `assets/models/fixtures/cube-obj/cube.mtl`: references `map_Kd texture.png`.

## Pending

- Screenshot validation is still pending; capture both fixtures before marking DONE.
