Place a `.gltf`, `.glb`, or `.obj` file here.

The app loads `models/model.gltf` by default. Pass another path as the first argument
if you want to open a different model.

Fixtures:

- `models/fixtures/box-textured/BoxTextured.gltf`: glTF with external `.bin` and `.png` sidecars.
- `models/fixtures/cube-obj/cube-tex.obj`: OBJ with `cube.mtl` and `texture.png`.
- `models/fixtures/fox-glb/Fox.glb`: animated GLB fixture for autoplay and Play/Pause testing.

Notes:

- Use GLB for animation testing in this app.
- OBJ is still useful here for geometry/material coverage, but not for clip playback. Blender's OBJ docs state that OBJ has no armature/animation support, and its "Animation" export mode writes a numbered OBJ per frame instead of a single animated asset: `https://docs.blender.org/manual/en/3.2/files/import_export/obj.html`
