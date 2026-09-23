---
name: graphics-pipeline
description: Orienta o desenvolvimento e a manutenção do pipeline gráfico do photorealism-plugin para Euro Truck Simulator 2 (motor Prism3D, Direct3D 11), um proxy dxgi.dll que intercepta Present, Present1 e ResizeBuffers e executa um passe fullscreen em HLSL de correção de cor e tonemapping com saída SDR, HDR10-PQ ou scRGB. Utilize esta skill ao implementar, modificar ou depurar shaders, efeitos de pós-processamento, configurações que alimentam o shader, hooks DXGI, estado D3D11 e ciclo de vida de recursos gráficos, ou ao avaliar se um efeito, como upscaling, é viável neste ponto de injeção.
---

# Graphics Pipeline

## Objetivo

Orientar alterações no pipeline gráfico do plugin preservando a estabilidade do jogo, a compatibilidade e a arquitetura do projeto.

Escopo validado: ETS2 no caminho Windows x64 com Direct3D 11 (README.md). O ATS não foi validado; não afirmar suporte a ele.

## Como o pipeline funciona hoje

Fluxo, em ordem de execução:

1. `src/proxy.rs` e `src/dxgi.def`: `dxgi.dll` exporta `CreateDXGIFactory`, `CreateDXGIFactory1` e `CreateDXGIFactory2`, encaminha ao DXGI do sistema e, com `force_hdr=true`, define `DXVK_HDR=1` antes dele carregar.
2. `src/hooks.rs` e `src/probe.rs`: uma swap chain descartável fornece a vtable. Os hooks entram em `Present` (índice 8), `ResizeBuffers` (13) e `Present1` (22), depois que `gameoverlayrenderer64.dll` estabiliza. Uma guarda por thread impede reentrância.
3. `src/runtime.rs`: singleton protegido por `try_lock`, que nunca bloqueia a thread de renderização. Relê a configuração a cada 1 s. `ResizeBuffers` chama `runtime::reset`, que descarta o `Renderer`.
4. `src/graphics/`: `Renderer` valida o backbuffer, copia-o para uma textura de origem (`CopyResource`) e desenha um triângulo fullscreen (`SV_VertexID`, sem vertex buffer) de volta no backbuffer, capturando e restaurando o estado do jogo.
5. `shaders/photorealism.hlsl`, embutido por `include_bytes!` e compilado em tempo de execução por `D3DCompile` (`d3dcompiler_47.dll`): decodifica, aplica exposição, balanço de branco, contraste, saturação e realces, e codifica.

Responsabilidades:

| Módulo | Responsabilidade |
| --- | --- |
| `graphics/resources.rs` | texturas, views, shaders, sampler, buffer, formatos suportados, `OutputMode` por formato |
| `graphics/state.rs` | captura e restauração do estado D3D11 |
| `graphics/constants.rs` | `ShaderSettings`, espelho `repr(C)` do constant buffer |
| `graphics/shaders.rs` | compilação do HLSL |
| `graphics/display.rs` | pico do monitor via `IDXGIOutput6::GetDesc1` |
| `color.rs`, `settings/`, `config/` | domínio puro, compilado também no host (`cfg(any(windows, test))`) e testável sem o jogo |

Modos de saída, decididos pelo formato do backbuffer: SDR (`R8G8B8A8` e `B8G8R8A8`, UNORM ou UNORM_SRGB; a conversão sRGB é manual só nos formatos UNORM), HDR10-PQ (`R10G10B10A2_UNORM`) e scRGB (`R16G16B16A16_FLOAT`, com 1.0 igual a 80 nits). MSAA, backbuffers menores que 640x480 e outros formatos são ignorados sem erro. A gradação ocorre nas primárias nativas de cada modo, sem conversão de gamut.

## Contratos que precisam permanecer sincronizados

- **Constant buffer:** `cbuffer SettingsBuffer` no HLSL, `ShaderSettings` em `graphics/constants.rs` e `EXPECTED_FIELDS` em `tests/shader_layout.rs`. Nenhum campo pode atravessar um registrador de 16 bytes e o total precisa fechar registradores inteiros. O teste compara o HLSL com a lista, não com a struct Rust; conferir a ordem da struct manualmente. Flags viajam como `float`.
- **Matemática espelhada:** `tests/tone_curve.rs` reimplementa o rolloff de realces em Rust e exige que o HLSL contenha a expressão exata do knee. Ao alterar essa expressão, atualizar o espelho e `KNEE_EXPRESSION`. Matemática nova de shader deve seguir o mesmo padrão.
- **Nova opção configurável:** campo, valor padrão e faixa de `clamp` em `Settings::apply`; teste em `settings/`; `config/photorealism-plugin.cfg`; `describe` em `runtime.rs`; README. Se chegar ao shader, entra também no contrato do constant buffer. `force_hdr` é a exceção: só vale na inicialização.
- **Estado D3D11:** `PipelineState` captura e restaura só o que o passe altera (shaders, IA, blend, depth, rasterizer, viewports e o slot 0 de render target, SRV, sampler e constant buffer). Todo estado ou slot novo tocado pelo passe deve entrar na captura e na restauração.
- **Ciclo de vida:** `FrameResources` guarda uma view do backbuffer, e `ResizeBuffers` falha se restar referência a ele. Por isso o hook libera o `Renderer` antes de chamar o original. Todo recurso novo que referencie o backbuffer deve ser liberado nesse mesmo caminho.
- **Thread de renderização:** o código roda dentro de `Present`. Não bloquear, não esperar, não registrar log por quadro. Erros são registrados uma vez (`error_reported`). Mensagens de log seguem o padrão existente: português sem acentos, via `logging::write`.

## O que a SCS e o Prism3D documentam

Verificado em 2026-09-23. Fontes na última seção.

- **Não há API gráfica oficial.** O Telemetry SDK 1.14 oferece só telemetria e dispositivos de entrada. Seus plugins são carregados de `bin/win_x64/plugins` ou de uma chave de registro, exportando `scs_telemetry_init` e `scs_telemetry_shutdown`, e não recebem device, swap chain nem quadros. Este plugin não usa esse mecanismo, e também não é um pacote de mod (sem `manifest.sii`, sem mod manager): é um `dxgi.dll` ao lado do executável.
- **O motor não é documentado.** A wiki de mods da SCS só tem as seções Engine e Tools, sem páginas sobre shaders, materiais, renderização ou internos do Prism3D, e a SCS não publica as APIs gráficas do motor. Passes internos, ordem de composição, formatos, resoluções e propriedade de recursos do jogo são desconhecidos: tratar como observação (log do plugin, `game.log.txt`, captura), nunca como fato.
- **Variáveis do `config.cfg`** (`Documents\Euro Truck Simulator 2` no Windows, `~/.local/share/Euro Truck Simulator 2` no Linux; console com `g_console` e `g_developer` iguais a 1 e comando `uset`):
  - `r_device` aceita `dx9`, `dx11` e `gl`. O plugin só atua em D3D11, daí `-rdevice dx11`. A wiki lista `dx9` como padrão e avisa que parte das páginas está obsoleta.
  - `r_color_correction`, `r_color_saturation`, `r_color_cyan_red`, `r_color_magenta_green` e `r_color_yellow_blue` são a correção de cor do próprio jogo, de natureza semelhante à de saturação, temperatura e tint do plugin. Ao avaliar o visual, registrar esses valores e não variar os dois lados ao mesmo tempo. `r_gamma` vale só em tela cheia, e o mecanismo que ele usa não é documentado.
  - `r_hdr`, `r_deferred`, `r_mlaa`, `r_dof`, `r_scale_x` e `r_scale_y` definem como o jogo produz a imagem; o plugin só observa o backbuffer no `Present`. A relação de `r_hdr` com a saída HDR de tela não está documentada.
- **HDR nativo desde a 1.57** (blog da SCS): ativado automaticamente em telas compatíveis, com calibração no menu. Na thread de suporte do fórum (12/11/2025), a equipe da SCS indica `-color_mode sdr` ou desativar o HDR no Windows para rodar em SDR e diz que o conteúdo é calibrado para HDR pela SCS, sem previsão de uso com modos automáticos ou adaptativos da tela. Usuários mostram a calibração gravada no `config.cfg` como `r_hdr_display_gray_offset`, `r_hdr_display_white` e `r_hdr_display_black`, variáveis ausentes da wiki e sem significado documentado. O plugin não lê essa calibração: `hdr_paper_white_nits` e o pico do DXGI são independentes dela, e em problemas de HDR convém registrar ambos. O `game.log.txt` lista os modos de cor detectados em "Detected color modes".
- **Consequência do ponto de injeção:** o passe recebe o quadro final, com interface e escala do jogo já aplicadas, sem profundidade, G-buffer, vetores de movimento ou passes intermediários. Efeitos espaciais sobre a imagem final são viáveis. Efeitos que dependem desses dados, como upscaling temporal e SSAO, não são viáveis neste ponto. Antes de propor upscaling espacial, verificar por captura a resolução e a composição do quadro no `Present`, porque isso não é documentado.

## Procedimento

### 1. Analisar a solicitação

- Identificar o objetivo e consultar `AGENTS.md`.
- Ler os módulos e o shader envolvidos e classificar a mudança: shader, contrato, estado D3D11, ciclo de vida, configuração ou hook.
- Separar o que é verificável no código do que depende de comportamento não documentado do jogo. Não presumir o segundo.

### 2. Planejar a implementação

- Criar uma branch separada para a alteração.
- Manter decisões e matemática em módulos puros, testáveis no host, e as chamadas D3D11 finas em `graphics/`.
- Identificar riscos de compatibilidade (Proton/DXVK, Steam Overlay, formatos de backbuffer) e de desempenho por quadro.
- Definir critérios de aceitação verificáveis.

### 3. Implementar a alteração

- Respeitar SRP e DIP, preservando abstrações como `ConfigSource`, que isola a leitura de arquivo e permite testes com fonte falsa.
- Funções pequenas, early return, `match`, `let else` e enums em vez de flags booleanas. Arquivos com no máximo 200 linhas, preferencialmente.
- Tratamento explícito de erros com `windows::core::Result`, sem `unwrap` ou `expect` no caminho de renderização.
- Não adicionar comentários ao código sem autorização.
- Não introduzir dependências: hoje só existe o crate `windows`.

### 4. Validar a implementação

No host e na compilação cruzada:

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo clippy --target x86_64-pc-windows-gnu -- -D warnings
cargo zigbuild --release --target x86_64-pc-windows-gnu
```

Só os comandos com alvo Windows compilam `graphics/`, `hooks`, `proxy` e `runtime`; `cargo test` e o clippy do host não os alcançam. Em 2026-09-23 o `cargo test` passava com 33 testes.

Shader:

- Os testes não compilam HLSL, só verificam layout e texto. A compilação real ocorre no jogo, e `shaders.rs` descarta o blob de erros do `D3DCompile`: uma falha de HLSL aparece no log apenas como "Falha ao inicializar o pipeline de cor e tonemap.", sem detalhe.
- Verificação auxiliar offline, quando `fxc` e `dxc` não existirem, com o front-end HLSL do glslang: `glslangValidator -D -V -S frag -e PSMain -o /dev/null shaders/photorealism.hlsl` e `glslangValidator -D -V -S vert -e VSMain -o /dev/null shaders/photorealism.hlsl`. Ele detecta erros de sintaxe e de símbolos, mas é outro compilador e não substitui o `d3dcompiler_47`.

No jogo, conferir `photorealism-plugin/photorealism-plugin.log`: "Hooks Present, Present1 e ResizeBuffers instalados.", "Pipeline de cor e tonemap inicializado." e "Backbuffer LxA formato=N modo=...". Mensagens de falha indicam o ponto que quebrou.

`tools/build.sh` incrementa a versão no `Cargo.toml` e publica em `dist/`. Usá-lo somente quando um pacote for pedido, nunca como validação.

### 5. Apresentar o resultado

Informar:

- Quais alterações foram realizadas e quais arquivos foram modificados.
- Quais testes foram executados e onde: host, compilação cruzada ou jogo.
- Quais problemas foram identificados.
- Quais limitações ainda existem, incluindo o que dependeu de comportamento não documentado do jogo.

Não afirmar que uma funcionalidade foi validada sem evidências dos testes correspondentes.

## Restrições técnicas

- Apenas Direct3D 11 (`ID3D11Device` obtido da swap chain). Não presumir D3D12, Vulkan ou OpenGL, nem suporte ao ATS.
- Não modificar estados gráficos sem avaliar seu impacto e sem restaurá-los.
- Não alterar recursos compartilhados com o jogo sem considerar seu ciclo de vida.
- Não atribuir ao Prism3D comportamentos que não foram observados.
- Não implementar funcionalidades fora do escopo solicitado.

## Critérios de conclusão

A tarefa somente poderá ser considerada concluída quando as alterações solicitadas estiverem implementadas, o projeto compilar sem novos warnings e os testes aplicáveis passarem.

Caso alguma verificação não possa ser executada, em especial a validação dentro do jogo, registrar explicitamente essa limitação.

## Fontes

- [SCS Modding Wiki: Documentation](https://modding.scssoft.com/wiki/Documentation)
- [Configuration variables](https://modding.scssoft.com/wiki/Documentation/Engine/Configuration_variables)
- [Console](https://modding.scssoft.com/wiki/Documentation/Engine/Console)
- [Game user path](https://modding.scssoft.com/wiki/Documentation/Engine/Game_user_path)
- [Mod manager](https://modding.scssoft.com/wiki/Documentation/Engine/Mod_manager)
- [Telemetry SDK](https://modding.scssoft.com/wiki/Documentation/Engine/SDK/Telemetry), com o `readme.txt` de `scs_sdk_1_14.zip`
- [SCS Software: Prism3D](https://www.scssoft.com/technology?lang=en)
- [ETS2 1.57: HDR](https://blog.scssoft.com/2025/11/euro-truck-simulator-2-157-update.html)
- [Fórum SCS: HDR setup](https://forum.scssoft.com/viewtopic.php?t=345222)
