# Visualizador de modelos

## Domínio

O visualizador abre um único GLB, glTF ou OBJ, centraliza o conteúdo e oferece inspeção e controles locais.

## Componentes

- [UI e renderização](UI_AND_RENDERING.md)
- [Modelo, animação e estatísticas](MODEL_AND_ANIMATION.md)

## Relações

Arquivo do modelo alimenta src/main.rs, que instancia WorldAssetRoot. A cena alimenta estatísticas, animações e modos de renderização; os três fluxos são exibidos e controlados por egui.
