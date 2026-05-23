# gltf-linux-preview

Simple glTF, GLB, and OBJ previewer for Linux, built with Bevy.

## Usage

1. Place your model in `assets/models/model.gltf`.
2. Run `cargo run`.
3. Or pass another path relative to `assets/`:

```bash
cargo run -- models/fox.glb
cargo run -- models/fixtures/fox-glb/Fox.glb
cargo run -- models/fixtures/cube-obj/cube-tex.obj
```

For animation testing, prefer GLB fixtures. OBJ support in this app is for static geometry/materials; Blender's OBJ docs note that animation export is a numbered OBJ sequence, not a single animated OBJ asset: `https://docs.blender.org/manual/en/3.2/files/import_export/obj.html`

## Install

### From a release

```bash
curl -fsSL https://github.com/allanbatista/gltf-linux-preview/releases/latest/download/install.sh | bash
```

To install into a different prefix:

```bash
curl -fsSL https://github.com/allanbatista/gltf-linux-preview/releases/latest/download/install.sh | bash -s -- --prefix /usr/local
```

The installer selects `x86_64` or `aarch64` automatically.

Releases are published automatically from `v*` tags by GitHub Actions.

### From source

```bash
./build-and-install.sh
```

By default, this installs to `~/.local` and registers the app so it appears in "Open With" for `.gltf`, `.glb`, and `.obj` files.
To use a different prefix:

```bash
PREFIX=/usr/local ./build-and-install.sh
```

After installation, you can also open a file directly:

```bash
gltf-linux-preview /path/to/model.glb
gltf-linux-preview /path/to/model.obj
```

## Controls

- Left mouse button drag: rotate the camera
- Scroll: zoom in and out
- The model spins continuously around the Y axis
