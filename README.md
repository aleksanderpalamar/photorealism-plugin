# Photorealism Plugin em Rust

Primeira versão de um plugin gráfico para Euro Truck Simulator 2, escrita em Rust e limitada a correção de cor e tonemapping.

O projeto usa a mesma estratégia básica da implementação de referência: `dxgi.dll` funciona como proxy do DXGI do sistema, instala hooks em `Present`, `Present1` e `ResizeBuffers` e executa um passe fullscreen em Direct3D 11 antes da apresentação do quadro. Não usa a Telemetry SDK nem uma API oficial do Prism3D.

## Escopo atual

- exposição;
- contraste;
- saturação;
- temperatura de cor;
- tint verde/magenta;
- compressão de realces;
- saída HDR10 com curva PQ e gamut Rec.2020 preservado;
- exposição automática do suporte HDR do DXVK ao jogo;
- saída scRGB linear com valores acima de 1.0 preservados;
- compatibilidade de fallback com SDR;
- reconstrução automática dos recursos após mudança de resolução;
- restauração do estado D3D11 usado pelo jogo antes do passe.

O shader é compilado em tempo de execução por `d3dcompiler_47.dll`, normalmente disponível no Windows e no Proton.

## Compilação

Dependências:

- Rust com o alvo `x86_64-pc-windows-gnu`;
- Zig;
- `cargo-zigbuild`;
- `mingw-w64-binutils`, que fornece `x86_64-w64-mingw32-dlltool`.

```bash
rustup target add x86_64-pc-windows-gnu
cargo install cargo-zigbuild --locked
./tools/build.sh
```

O pacote será criado em uma pasta e um arquivo ZIP versionados, como `dist/photorealism-plugin-0.2.1-ets2-hdr/` e `dist/photorealism-plugin-0.2.1-ets2-hdr.zip`. Versões anteriores não são removidas automaticamente.

## Instalação

Copie estes itens para `Euro Truck Simulator 2/bin/win_x64/`:

```text
dxgi.dll
photorealism-plugin/
```

Não mantenha outro proxy chamado `dxgi.dll` no mesmo diretório. Faça backup do proxy existente antes de testar.

No Proton, pode ser necessário definir a substituição:

```text
WINEDLLOVERRIDES="dxgi=n,b"
```

No GNOME Wayland com GE-Proton, use nas opções de inicialização do ETS2:

```text
PROTON_ENABLE_WAYLAND=1 PROTON_ENABLE_HDR=1 WINEDLLOVERRIDES="dxgi=n,b" %command% -rdevice dx11
```

Se iniciar o jogo dentro de uma instância própria do Gamescope, use:

```text
PROTON_ENABLE_HDR=1 WINEDLLOVERRIDES="dxgi=n,b" gamescope --hdr-enabled -f -- %command% -rdevice dx11
```

As opções são lidas de `photorealism-plugin/photorealism-plugin.cfg` na inicialização. Reinicie o jogo depois de alterá-las. O log é gravado em `photorealism-plugin/photorealism-plugin.log`.

`force_hdr=true` define `DXVK_HDR=1` antes de o DXGI real ser carregado, permitindo que o ETS2 detecte os modos HDR no Proton. Essa opção não substitui `PROTON_ENABLE_HDR=1`, que precisa existir antes de o Proton iniciar. Para HDR, ajuste `hdr_paper_white_nits` para o nível de branco confortável da tela e `hdr_peak_nits` para o pico calibrado do monitor. O valor inicial usa 203 nits para paper white e 1000 nits para o pico. O log informa `HDR10-PQ`, `scRGB` ou `SDR` conforme o backbuffer criado pelo jogo.

O HDR deve estar habilitado no sistema e detectado pelo ETS2. Ao executar o jogo dentro de uma instância própria do Gamescope, inclua `--hdr-enabled`. O suporte nativo do jogo foi introduzido na versão 1.57.

## Desenvolvimento

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo clippy --target x86_64-pc-windows-gnu -- -D warnings
```

A DLL foi projetada exclusivamente para ETS2 no caminho Windows x64 com Direct3D 11. A validação dentro do jogo ainda deve ser feita antes de considerar esta versão pronta para distribuição ampla.
