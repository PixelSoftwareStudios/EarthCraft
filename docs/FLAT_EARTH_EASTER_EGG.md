# Hidden flat-Earth illustration / Ilustração secreta da Terra plana

This is a **visual Easter egg** based on the image provided by the user. It does
**not** change EarthCraft's globe, geodesy, camera coordinates, live aircraft,
weather/cloud maps, or other data sources. The selected scientific Earth model remains unchanged.

Esta é uma **brincadeira visual** baseada na imagem fornecida pelo usuário.
Ela **não** altera o modelo científico da Terra, suas coordenadas, os aviões reais, as
nuvens, os mapas meteorológicos ou quaisquer fontes de dados.

## Como encontrar / How to activate

- Pressione **Ctrl + Shift + F** no Windows/Linux, ou **⌘ + Shift + F** no macOS.
- Ou digite **terra plana** na busca do EarthCraft e pressione Enter ou Buscar.
- English alternative: search for **flat earth**.

## Controles / Controls

- Arraste com o mouse para mover a ilustração / Drag to pan.
- Use a roda do mouse para ampliar / Scroll to zoom.
- Dê dois cliques para restaurar tamanho e posição / Double-click to reset.
- Pressione **Esc**, repita o atalho, use **Voltar ao globo**, ou selecione
  Terra/Lua/Planeta personalizável para sair / Exit with Esc, the shortcut,
  the return button or by selecting a regular celestial body.

## Arquivos envolvidos / Files

- `apps/earthcraft/assets/flat_earth_easter_egg.png` — imagem incorporada ao aplicativo.
- `apps/earthcraft/src/app/flat_earth.rs` — lógica de ativação e visualização.
- `apps/earthcraft/src/app/mod.rs` — integração com o estado e a tela principal.
- `apps/earthcraft/src/app/search.rs` — frase secreta na pesquisa.
- `apps/earthcraft/src/app/ui/bars.rs` e `apps/earthcraft/src/app/explorer.rs` — retorno aos corpos reais.
- `crates/earthcraft-core/locales/{en,pt-BR}.ftl` — textos em ambos os idiomas.

The image is displayed for illustration/humor, not as a scientific depiction.

## 3D estrelado (atualização)

A textura equiretangular é amostrada por uma malha polar de 96 setores × 32 anéis,
projetada em perspectiva com rotação horizontal e inclinação vertical. Um
anel lateral cria a espessura do disco e 340 estrelas procedurais compõem o
fundo. Arrastar gira; Shift+arrastar desloca; roda amplia; duplo clique reinicia.
É uma paródia 3D, nunca uma substituição da projeção do globo científico.

## Cached scene / Cena em cache

The illustration stays outside the Earth-model selector. Its image is decoded
on the file worker, and completing a load after exit does not reopen the scene.
The polar disk shares 3,201 vertices across its unchanged 6,144 triangles.
Stars, rim, disk and ice-outline tessellation are cached until the viewport,
rotation, tilt, zoom, pan, texture or DPI changes. Labels and controls remain
interactive. The scientific globe and live services do not use this geometry.

A ilustração continua fora do seletor de modelos. A imagem é decodificada pelo
serviço de arquivos; concluir uma carga após sair não reabre a cena. O disco
compartilha 3.201 vértices em seus mesmos 6.144 triângulos, com malhas em cache
até mudar a vista, rotação, inclinação, zoom, posição, textura ou DPI.
