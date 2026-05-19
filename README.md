# gltf-linux-preview

Previewador simples de modelos glTF em Bevy para Linux.

## Uso

1. Coloque seu modelo em `assets/models/model.gltf`.
2. Rode `cargo run`.
3. Ou passe outro caminho relativo a `assets/`:

```bash
cargo run -- models/fox.glb
```

## Controles

- Botao esquerdo e arraste: rotaciona a camera
- Scroll: zoom in/out
- O modelo gira continuamente no eixo Y
