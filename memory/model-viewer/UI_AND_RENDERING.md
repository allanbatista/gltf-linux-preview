# UI e renderização

## Responsabilidade

Exibir controles egui e aplicar o modo visual, a rotação automática, a iluminação e a thumbnail.

## Entidades

- ViewSettings mantém o modo base e a sobreposição de wireframe.
- AutoRotation controla ModelSpinner.
- ThumbnailTarget guarda a imagem estática gerada pela câmera temporária.
- MainCamera, OrbitCamera e CameraLight mantêm a luz alinhada à câmera.

## Relações

- draw_ui atualiza ViewSettings, AutoRotation e AnimationPlayback.
- apply_view_settings restaura materiais ou aplica variantes unlit/sólidas e Wireframe quando a GPU suporta os recursos necessários.
- sync_camera_light copia a transformação da câmera principal.

## Fluxo

1. A câmera enquadra o modelo.
2. Uma câmera secundária renderiza a thumbnail e é removida após a captura.
3. egui desenha os painéis superiores e os modos na base.
4. A alteração do modo é aplicada no próximo ciclo de atualização.

## Fontes no código

- src/main.rs
- Cargo.toml
