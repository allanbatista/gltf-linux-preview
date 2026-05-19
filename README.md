# gltf-linux-preview

Simple glTF and GLB previewer for Linux, built with Bevy.

## Usage

1. Place your model in `assets/models/model.gltf`.
2. Run `cargo run`.
3. Or pass another path relative to `assets/`:

```bash
cargo run -- models/fox.glb
```

## Install

```bash
./build-and-install.sh
```

By default, this installs to `~/.local` and registers the app so it appears in "Open With" for `.gltf` and `.glb` files.
To use a different prefix:

```bash
PREFIX=/usr/local ./build-and-install.sh
```

After installation, you can also open a file directly:

```bash
gltf-linux-preview /path/to/model.glb
```

## Controls

- Left mouse button drag: rotate the camera
- Scroll: zoom in and out
- The model spins continuously around the Y axis
