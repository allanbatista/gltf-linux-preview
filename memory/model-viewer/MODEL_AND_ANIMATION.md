# Modelo, animação e estatísticas

## Responsabilidade

Carregar o ativo, enquadrar sua geometria, calcular estatísticas e reproduzir os clipes glTF.

## Entidades

- ModelAssets guarda os handles de Gltf e WorldAsset quando aplicáveis.
- ModelMesh preserva os handles originais de malha e material.
- ModelStats contém polígonos/faces triangulados, vértices e VRAM estimada.
- AnimationPlayback mantém clipe selecionado, pausa e loop.

## Relações

- WorldAssetRoot instancia a cena do GLB/glTF/OBJ.
- O modo headless aguarda todas as dependências de WorldAsset antes de capturar o PNG.
- center_pending_model calcula AABB, estatísticas e enquadramento.
- setup_animation_graph cria um grafo com todos os clipes do glTF.
- draw_ui seleciona ou pausa a reprodução em AnimationPlayer.

## Fluxo

1. O loader abre o ativo usando caminho aprovado explicitamente.
2. A cena instancia malhas e materiais.
3. O visualizador centraliza a AABB, conta polígonos/faces triangulados, vértices e soma buffers/texturas únicos.
4. O primeiro clipe disponível inicia em loop; OBJ ou modelos estáticos exibem ausência de animações.

## Fontes no código

- src/main.rs
- assets/models/README.md
