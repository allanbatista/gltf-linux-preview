# gltf-linux-preview

Visualizador Linux para arquivos glTF, GLB e OBJ, feito com Bevy 0.19 e egui.

## Uso

1. Coloque o modelo em assets/models/model.gltf.
2. Execute:

    cargo run

Também é possível informar um caminho relativo a assets:

    cargo run -- models/fixtures/fox-glb/Fox.glb
    cargo run -- models/fixtures/box-textured/BoxTextured.gltf
    cargo run -- models/fixtures/cube-obj/cube-tex.obj

Use GLB para testar animações. OBJ é aceito para geometria e materiais estáticos, mas não contém animações.

## Interface

- **Modelo:** thumbnail estática, polígonos/faces triangulados, vértices e VRAM estimada.
- **Rotação:** inicia ligada e pode ser ativada ou desativada no painel do modelo.
- **Animações:** seleciona qualquer clipe, inicia pelo primeiro em loop e permite pausar ou desativar o loop.
- **Visualização:** os botões centralizados na base alternam entre Renderizado, Texturizado, Suave e Sólido. Wireframe é uma sobreposição para Texturizado, Suave e Sólido quando a GPU o suporta.

A VRAM é uma estimativa dos buffers de vértices/índices e das texturas únicas do modelo. Não inclui a thumbnail nem caches transitórios da visualização.

## Controles da cena

- Arraste com o botão esquerdo: orbita a câmera.
- Roda do mouse: aproxima ou afasta.
- A luz direcional acompanha a câmera e aponta para o centro do modelo.

## Instalação

### Release

    curl -fsSL https://github.com/allanbatista/gltf-linux-preview/releases/latest/download/install.sh | bash

Para outro prefixo:

    curl -fsSL https://github.com/allanbatista/gltf-linux-preview/releases/latest/download/install.sh | bash -s -- --prefix /usr/local

O instalador seleciona automaticamente x86_64 ou aarch64. Releases são publicadas a partir de tags v*.

### Código-fonte

    ./build-and-install.sh

Por padrão, o instalador usa ~/.local e registra o app para .gltf, .glb e .obj.

Para outro prefixo:

    PREFIX=/usr/local ./build-and-install.sh

Após a instalação:

    gltf-linux-preview /caminho/para/modelo.glb

### Thumbnails no gerenciador de arquivos

O instalador registra um thumbnailer XDG para GLB, glTF e OBJ. O gerador também pode ser chamado diretamente:

    gltf-linux-preview --thumbnail modelo.glb thumbnail.png 256

No GNOME/Nautilus, o thumbnailer roda em sandbox e só enxerga executáveis instalados em `/usr`. A instalação padrão em `~/.local` mantém a associação de arquivos, mas não consegue gerar thumbnails automáticas. Para habilitá-las no GNOME, instale o app no sistema:

    sudo env PREFIX=/usr ./build-and-install.sh

Para a release, use `sudo env PREFIX=/usr` ao executar `install.sh`. Se `~/.local/share/thumbnailers` for criado durante a sessão, execute `nautilus -q` antes de abrir a pasta novamente.

O sandbox do GNOME fornece somente o arquivo selecionado: GLB funciona por ser autocontido; glTF e OBJ que dependem de `.bin`, `.mtl` ou texturas ao lado do arquivo não podem ser renderizados automaticamente nesse ambiente.
