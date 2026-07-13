# Visualizador de modelos

## Domínio

O visualizador abre um único GLB, glTF ou OBJ, centraliza o conteúdo e oferece inspeção e controles locais.

## Componentes

- [UI e renderização](UI_AND_RENDERING.md)
- [Modelo, animação e estatísticas](MODEL_AND_ANIMATION.md)
- [Geração de thumbnails](THUMBNAILING.md)

## Relações

Arquivo do modelo alimenta src/main.rs, que instancia WorldAssetRoot. A cena alimenta estatísticas, animações e modos de renderização; esses fluxos são exibidos e controlados por egui. O mesmo carregamento também produz PNGs para thumbnailers XDG em modo headless.
