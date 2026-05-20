# OBJ/MTL support and external texture loading

Status: READY_FOR_PLAN

## Goal

Let Linux users preview common Wavefront OBJ models with MTL materials in the existing viewer, and make glTF JSON files reliably show their externally referenced textures. The outcome is that a user can open a local `.obj`, `.gltf`, or `.glb` file from the command line or desktop "Open With" flow and see the model centered, lit, textured, spinning, and controllable with the existing orbit/zoom interactions.

This feature is about practical model preview fidelity, not a full DCC importer. The viewer should preserve the current fast "open a file and inspect it" workflow while covering the common asset layouts users download from marketplaces, scanners, CAD conversion tools, and sample repositories.

## Users & Journeys

- **Linux desktop user opening a downloaded OBJ:** From the file manager or terminal, the user opens `model.obj`. The viewer loads geometry, applies the referenced `.mtl` material library, displays diffuse/base-color texture maps where present, falls back to material colors where textures are absent, and keeps the current camera controls.
- **Developer or artist checking a glTF with sidecar images:** From the terminal, the user opens `/path/to/asset/scene.gltf` where images are referenced by relative URI such as `textures/baseColor.png`. The viewer resolves those images relative to the `.gltf` file, displays them on the model, and does not require copying the model into the repository `assets/` directory.
- **User with incomplete assets:** If a top-level file is missing or invalid, the user gets a readable error naming the file and the app exits cleanly. If a referenced MTL or texture is missing, the viewer still shows any loadable geometry with a neutral fallback material and logs a warning naming the missing referenced file.
- **Existing glTF/GLB user:** Existing `.gltf` and `.glb` workflows continue to work, including the default `models/model.gltf` behavior, command-line file opening, centering, spin, orbit, and zoom.

## Product Inventory

| Route/Page/Surface | Slug/ID | Label | Output type | Inputs and filters | Required datasets/permissions | Empty/loading/error/locked behavior | Persona differences |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Viewer window launched without an argument | `default-model` | GLTF Preview | Interactive 3D scene | Default local model path `models/model.gltf` | Read access to packaged app assets | Loading may show the existing background until the model appears; missing default file exits cleanly with a readable error | Same for all users |
| Viewer window launched with a glTF binary file | `open-glb` | GLTF Preview | Interactive 3D scene | `.glb` local file path, absolute or relative | Read access to selected file | Invalid or missing file exits cleanly with a readable error | Same for all users |
| Viewer window launched with a glTF JSON file | `open-gltf` | GLTF Preview | Interactive 3D scene | `.gltf` local file path, absolute or relative; relative external buffers and PNG/JPEG images referenced by the glTF file | Read access to selected file and referenced local sidecar files | Missing sidecar textures log warnings and show fallback material; missing required model/buffer data exits cleanly with a readable error | Same for all users |
| Viewer window launched with a Wavefront OBJ file | `open-obj` | GLTF Preview | Interactive 3D scene | `.obj` local file path, absolute or relative; `mtllib` material libraries; `map_Kd` PNG/JPEG/TGA diffuse textures | Read access to selected file and referenced local MTL/texture files | Missing MTL or texture logs warnings and shows loadable geometry with fallback material; invalid top-level OBJ exits cleanly with a readable error | Same for all users |
| Desktop "Open With" integration after install | `desktop-open-with` | GLTF Preview | Opens selected file in viewer | `.gltf`, `.glb`, and `.obj` files exposed by the Linux desktop file manager | Installed desktop entry and read access to selected file | If the desktop environment cannot identify OBJ MIME, command-line `.obj` opening remains supported and documented | Same for all users |

## Non-Functional Requirements

- **Security:** Only local files explicitly selected by the user or referenced by that selected local model may be read. Remote `http://` and `https://` assets must not be fetched for OBJ/MTL or glTF texture loading. Missing or blocked references must produce readable warnings instead of panics.
- **Path handling:** File paths and relative asset references must work with nested folders, spaces, and URI-encoded spaces. Relative references resolve from the file that declares them: glTF external resources from the `.gltf` directory, OBJ `mtllib` from the `.obj` directory, and MTL texture maps from the `.mtl` directory.
- **Performance:** Typical preview assets up to 100k triangles with up to four 4096px textures should open without crashing on a standard Linux desktop. After loading, orbit and zoom interactions should remain responsive enough for inspection.
- **Compatibility:** Existing `.gltf` and `.glb` behavior must not regress. GLB embedded textures and data-URI glTF textures remain supported.
- **Accessibility and UX:** Errors and warnings must be readable in the terminal or application logs, include the affected path, and avoid stack traces for expected user-facing failures. Existing mouse controls remain unchanged.
- **Platform:** Linux desktop remains the supported platform. No mobile, web, or non-Linux packaging behavior is required.

## Acceptance Criteria

- **AC-01:** Given `gltf-linux-preview /tmp/model.obj` where `model.obj` contains vertices, normals, UVs, faces, `mtllib model.mtl`, and `usemtl`, the viewer displays the OBJ geometry centered in the scene with the existing spin, orbit, and zoom behavior.
- **AC-02:** Given an OBJ that references an MTL with a material color but no texture, the viewer applies the material color or a visibly distinct fallback material and logs no missing-texture error for that material.
- **AC-03:** Given an OBJ whose MTL uses `map_Kd textures/diffuse.png`, where the texture exists relative to the `.mtl` file, the viewer displays that diffuse texture on the model.
- **AC-04:** Given OBJ/MTL sidecar paths containing spaces, including a texture reference encoded as `textures/paint%20diffuse.jpg`, the viewer loads the referenced material or texture and renders it without requiring the user to rename files.
- **AC-05:** Given an OBJ that references a missing MTL file, the viewer still displays loadable geometry with a neutral fallback material and emits a warning that includes the missing MTL path.
- **AC-06:** Given an OBJ whose MTL references a missing texture file, the viewer displays the geometry with a fallback material for the affected surface and emits a warning that includes the missing texture path.
- **AC-07:** Given an invalid or unreadable top-level `.obj` path, the app exits cleanly with a non-zero status and a readable error naming the requested OBJ file.
- **AC-08:** Given `/tmp/asset/scene.gltf` with external PNG/JPEG image URIs in the same folder or child folders, the viewer displays those textures without requiring the asset folder to be copied under the repository `assets/` directory.
- **AC-09:** Given a glTF texture URI containing spaces or URI-encoded spaces, the viewer resolves it relative to the `.gltf` file and displays the texture.
- **AC-10:** Given a `.gltf` with embedded GLB-equivalent data URIs or an existing `.glb`, previously supported embedded geometry and textures still render.
- **AC-11:** Given a glTF or OBJ file that references `http://` or `https://` texture URLs, the viewer does not fetch remote content and instead reports an unsupported external reference in a readable warning or error.
- **AC-12:** Given installation through the existing install flow, the resulting user-facing documentation and desktop integration advertise `.obj` support alongside `.gltf` and `.glb` support where the Linux desktop environment supports OBJ file association.
- **AC-13:** Given the existing default no-argument launch path, the viewer still attempts to open `models/model.gltf` and preserves current camera centering, spin, orbit, zoom, windowed desktop behavior, and visual background.
- **AC-14:** Given automated or manual validation assets for supported OBJ/MTL and glTF external texture cases, the project has observable evidence through tests, logs, screenshots, or documented manual checks for every acceptance criterion above.

## Scope

In scope:

- Opening `.obj` files through the same viewer entry points used for glTF/GLB.
- Reading Wavefront MTL material libraries referenced by OBJ files.
- Rendering diffuse/base-color material textures from MTL `map_Kd` entries.
- Rendering material color/fallback appearance when texture data is absent or incomplete.
- Resolving local relative sidecar assets for OBJ, MTL, and glTF JSON files.
- Updating user-facing docs and install/desktop copy so supported formats are clear.
- Preserving existing glTF/GLB behavior and viewer controls.

Out of scope:

- Editing, exporting, converting, or saving models.
- A file picker, drag-and-drop UI, progress bar, model tree, inspector, or material editor.
- Full MTL/PBR parity for every legacy material property.
- Animation, skeletal data, morph targets, or cameras/lights from OBJ files.
- Network asset fetching.
- Windows, macOS, web, or mobile packaging.
- Renaming the application or changing the core viewer interaction model.

## Boundaries

- This spec is product/UX only. Implementation details, code structure, dependencies, files, schemas, and task order belong in `plan.md`, not here.
- Future source changes must follow the project workflow gate: `spec.md` to `plan.md` to `progress.md`.
- Keep the feature small: add practical local OBJ/MTL preview and fix local glTF external textures without expanding into a general asset-management tool.
- Do not remove or degrade the existing no-argument default model behavior, command-line launch behavior, installed launcher behavior, or current camera controls.
- If implementation discovers a format limitation that would reduce any acceptance criterion, the spec must return to `DRAFT` or be amended before planning proceeds.

## Open Questions

None blocking for product intent. Planning may choose the implementation approach, validation asset locations, and exact Linux OBJ MIME metadata as long as the acceptance criteria remain observable.

## Definition of Done

- Every acceptance criterion has passing observable evidence through automated tests, manual checks with recorded commands/logs/screenshots, or both.
- At least one OBJ fixture with MTL color, one OBJ fixture with `map_Kd` texture, one OBJ fixture with paths containing spaces, and one glTF fixture with external texture sidecars are validated.
- Existing `.gltf` and `.glb` launch paths are validated for non-regression.
- User-facing README/install text reflects `.obj`, `.mtl`, and sidecar texture expectations.
- The final handoff is gated by `e2e-validator` or equivalent end-to-end validation evidence before the feature is marked complete.
