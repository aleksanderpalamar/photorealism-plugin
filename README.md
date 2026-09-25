# Photorealism Plugin em Rust

Primeira versão de um plugin gráfico para Euro Truck Simulator 2, escrita em Rust e limitada a correção de cor e tonemapping.

O projeto usa a mesma estratégia básica da implementação de referência: `dxgi.dll` funciona como proxy do DXGI do sistema, instala hooks em `Present`, `Present1` e `ResizeBuffers` e executa um passe fullscreen em Direct3D 11 antes da apresentação do quadro. Não usa a Telemetry SDK nem uma API oficial do Prism3D.

## Escopo atual

- exposição;
- perfil de luminância global, em três níveis;
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
- restauração do estado D3D11 usado pelo jogo antes do passe;
- recarga da configuração em tempo de execução, sem reiniciar o jogo;
- detecção do pico de brilho do monitor pelo DXGI;
- luminância calculada nas primárias do espaço de saída, Rec.709 ou Rec.2020;
- painel em tela com os valores atuais, aberto e fechado por `CTRL+P`.

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

Cada execução publica uma versão nova. O script parte da maior versão que encontra entre o `Cargo.toml`, os pacotes em `dist/` e as tags `v*` do git, e incrementa a partir dela:

```bash
./tools/build.sh            # 0.2.1 -> 0.2.2
./tools/build.sh minor      # 0.2.1 -> 0.3.0
./tools/build.sh major      # 0.2.1 -> 1.0.0
./tools/build.sh --dry-run  # mostra a versão calculada sem alterar nada
```

A versão escolhida é gravada no `Cargo.toml` antes dos testes, do clippy e da compilação. Se qualquer uma dessas etapas falhar, `Cargo.toml` e `Cargo.lock` voltam ao estado anterior e nada é publicado em `dist/`. O commit da nova versão fica por sua conta; o script sugere a linha ao terminar.

O pacote é criado em uma pasta e um arquivo ZIP versionados, como `dist/photorealism-plugin-0.2.5/` e `dist/photorealism-plugin-0.2.5.zip`. Versões anteriores não são removidas nem sobrescritas.

Pacotes gerados até a 0.2.4 levavam o sufixo `-ets2-hdr` no nome. O cálculo da próxima versão continua reconhecendo esse formato antigo, para que a numeração não retroceda por causa deles.

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

As opções são lidas de `photorealism-plugin/photorealism-plugin.cfg`. Com o jogo aberto, o arquivo é relido a cada segundo e os novos valores entram em vigor no quadro seguinte, sem reinício. Cada recarga é registrada em `photorealism-plugin/photorealism-plugin.log` com os valores aplicados. A única exceção é `force_hdr`, que precisa existir antes de o DXGI ser carregado e só vale a partir da próxima abertura do jogo; o log avisa quando essa opção é alterada em tempo de execução.

Se o arquivo estiver ausente ou ilegível no momento da leitura, os últimos valores válidos continuam em uso.

`force_hdr=true` define `DXVK_HDR=1` antes de o DXGI real ser carregado, permitindo que o ETS2 detecte os modos HDR no Proton. Essa opção não substitui `PROTON_ENABLE_HDR=1`, que precisa existir antes de o Proton iniciar. Para HDR, ajuste `hdr_paper_white_nits` para o nível de branco confortável da tela; o valor inicial é 203 nits.

### Perfil de luminância

`luminance_profile` aceita `low`, `neutral` ou `high`, e o padrão é `neutral`. O perfil **soma** um deslocamento de exposição sobre o valor de `exposure`, em vez de substituí-lo: a calibração fina que você já fez continua valendo ao trocar de perfil.

| Perfil | Painel | Deslocamento | Cinza médio | Branco difuso equivalente |
| --- | --- | --- | --- | --- |
| `low` | `baixa` | −0,50 EV | 25,8 nits | 143 nits |
| `neutral` | `neutra` | 0,00 EV | 36,5 nits | 203 nits |
| `high` | `alta` | +0,50 EV | 51,6 nits | 287 nits |

Os valores não são preferência: o perfil neutro é o branco difuso de 203 nits da ITU-R BT.2408, que é também o padrão de `hdr_paper_white_nits`, e o pivô de contraste do shader em 0,18 põe o cinza médio em 36,5 nits — o cinza de 18% da fotografia. O degrau de meio stop é o bracket fotográfico clássico e mantém o branco difuso dentro da faixa que produções HDR reais usam.

Como `neutral` soma zero, uma configuração sem essa chave se comporta exatamente como antes de o perfil existir.

A soma é recortada na mesma faixa de `exposure`, de −4 a +4 EV. Com `exposure` além de ±3,5 o perfil perde efeito, parcial ou totalmente — é o limite de segurança, não um defeito. O log traz `profile=` e `exposure_eff=` para mostrar o valor que de fato chegou ao shader.

`hdr_peak_nits` aceita `auto`, que é o padrão, ou um número entre 400 e 10000. Em `auto` o plugin consulta `IDXGIOutput6::GetDesc1` e usa o `MaxLuminance` informado pelo monitor, limitado à mesma faixa. Como nem todo caminho DXVK/Wayland preenche esse campo, o log registra o que foi lido: se o valor vier ausente ou abaixo de 250 nits, ele é descartado e o plugin volta para 1000 nits. Compare o número do log com a especificação do seu monitor e, se não bater, fixe o valor correto no lugar de `auto`. O log informa `HDR10-PQ`, `scRGB` ou `SDR` conforme o backbuffer criado pelo jogo.

Saturação e balanço de branco dependem do peso de luminância de cada canal. Em HDR10-PQ o conteúdo está em Rec.2020 e os pesos usados são os dessa norma; em SDR e scRGB valem os de Rec.709. A gradação continua acontecendo nas primárias nativas de cada modo, sem conversão de gamut, para não descartar as cores fora do Rec.709 que o HDR10 carrega. Por isso `temperature` e `tint` têm força um pouco diferente entre os modos e merecem calibração separada.

O HDR deve estar habilitado no sistema e detectado pelo ETS2. Ao executar o jogo dentro de uma instância própria do Gamescope, inclua `--hdr-enabled`. O suporte nativo do jogo foi introduzido na versão 1.57.

## Painel em tela

`CTRL+P` abre e fecha um painel no canto superior esquerdo com os parâmetros e seus valores atuais. Cada mudança de estado é registrada no log.

Com o painel aberto, arraste os controles deslizantes com o mouse e clique nas caixas para alternar `Ativado` e `Pico automatico`. A linha `Perfil de luminancia` é uma lista: clicar na caixa abre as opções sobre as linhas de baixo, clicar em uma delas escolhe e fecha, e clicar fora fecha sem mudar nada. Os valores entram em vigor no quadro seguinte. A linha do pico fica esmaecida enquanto `hdr_peak_nits=auto`, porque nesse modo o valor vem do monitor e não da configuração; ela mostra `auto` seguido do valor que está em vigor, como `auto 1499`.

Enquanto o painel está aberto, mouse e teclado deixam de chegar ao jogo: o plugin intercepta `GetDeviceState` e `GetDeviceData` do DirectInput, lê os deslocamentos do mouse para mover o próprio ponteiro e devolve o estado zerado. Sem isso, arrastar um controle também giraria a câmera. Volantes, pedais e outros dispositivos não são bloqueados. O log confirma a instalação e registra, no primeiro bloqueio, se o jogo lê a entrada por estado ou por buffer.

Cada linha tem um botão `R` que devolve aquele parâmetro ao padrão. No rodapé, `Salvar no cfg` grava o estado atual no arquivo e `Descartar` volta ao que está gravado. O botão de salvar fica destacado enquanto houver alteração pendente.

Enquanto nada for arrastado, o painel continua seguindo o arquivo de configuração — ele permanece útil como leitura ao vivo. A partir do primeiro ajuste, os valores do painel passam a valer sobre o arquivo até você salvar ou descartar.

A gravação é feita em um arquivo temporário e renomeada por cima do original, para que uma falha no meio da escrita não deixe a configuração truncada. Depois de salvar, a releitura periódica encontra os mesmos valores e registra no log o que foi gravado.

O painel é desenhado depois do passe de cor, e suas cores passam pela mesma codificação de saída — PQ em HDR10, escala por paper white em scRGB, sRGB em SDR. Por isso ele aparece com o mesmo brilho de referência em qualquer um dos três modos, em vez de estourar em HDR.

Se `enabled=false`, o passe de cor não roda mas o painel continua podendo ser aberto.

Se a inicialização do pipeline falhar, o plugin registra a falha uma vez e não tenta de novo até uma troca de resolução ou de modo de tela, que é quando `ResizeBuffers` descarta o estado. Sem isso, cada quadro recompilaria os quatro shaders e gravaria o erro no log.

## Desenvolvimento

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo clippy --target x86_64-pc-windows-gnu -- -D warnings
```

A DLL foi projetada exclusivamente para ETS2 no caminho Windows x64 com Direct3D 11. A validação dentro do jogo ainda deve ser feita antes de considerar esta versão pronta para distribuição ampla.
