# Geração de thumbnails

## Responsabilidade

Produzir PNGs de preview para thumbnailers XDG sem criar uma janela nem depender de X11 ou Wayland.

## Entidades

- ThumbnailConfig representa `--thumbnail ENTRADA SAIDA.png TAMANHO`.
- HeadlessThumbnailState controla o enquadramento, a captura e a falha do render offscreen.
- ThumbnailTarget é a imagem de renderização usada por Screenshot.
- PendingThumbnailCentering indica que o WorldAsset ainda precisa ser enquadrado.

## Relações

- `install.sh` e `build-and-install.sh` registram `gltf-linux-preview.thumbnailer` para GLB, glTF e OBJ.
- `build-and-install.sh` reutiliza `target/release/gltf-linux-preview` se `cargo` não estiver no PATH, permitindo instalar em `/usr` após o build do usuário.
- O thumbnailer chama o binário com `%i`, `%o` e `%s`.
- `run_thumbnail` desabilita Winit, renderiza para ThumbnailTarget e grava Screenshot como PNG.
- `ModelAssets.scene` impede a captura antes de todas as dependências carregarem.

## Fluxo

1. O gerenciador de arquivos chama `--thumbnail` com o arquivo de entrada, PNG de saída e tamanho.
2. O app cria uma cena e câmera offscreen, aguarda WorldAsset e suas dependências e enquadra a AABB.
3. Após dois frames, Screenshot lê o render target e grava o PNG.
4. No GNOME/Nautilus, o helper precisa estar em `/usr`; o sandbox só fornece o arquivo principal, portanto GLB autocontido é suportado automaticamente e glTF/OBJ com arquivos externos não são.

## Fontes no código

- src/main.rs
- install.sh
- build-and-install.sh
- README.md
