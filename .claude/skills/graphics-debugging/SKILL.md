---
name: graphics-debugging
description: Diagnostica e orienta a correção de crashes, artefatos visuais, cores erradas, ausência de efeito, falhas de inicialização e regressões de desempenho no photorealism-plugin para Euro Truck Simulator 2 (proxy dxgi.dll em Direct3D 11, com passe HLSL de correção de cor e tonemapping em SDR, HDR10-PQ e scRGB, também sob Proton e DXVK). Use esta skill ao investigar hooks que não instalam, erros de shader, estado D3D11 corrompido, problemas no ResizeBuffers, HDR incorreto ou queda de FPS, a partir dos logs do plugin, do jogo e do ambiente.
---

# Graphics Debugging

## Objetivo

Orientar a investigação e a correção de problemas no plugin gráfico com um processo baseado em evidências, isolamento de causas e validação de resultados. Priorizar a estabilidade do jogo, a integridade dos recursos gráficos e a causa raiz.

Arquitetura, contratos entre HLSL e Rust e comandos de validação estão na skill `graphics-pipeline`. Escopo validado: ETS2 com Direct3D 11; o ATS não foi validado.

## Evidências disponíveis

### Log do plugin

`photorealism-plugin/photorealism-plugin.log`, ao lado do `dxgi.dll` (`bin/win_x64/`). O arquivo só recebe acréscimos: não tem data nem hora, e execuções diferentes se misturam. Uma sessão costuma começar por uma das linhas de `prepare_dxgi`. Erros de gravação são ignorados, então log ausente significa DLL não carregada ou pasta sem escrita. Nada é registrado por quadro.

| Mensagem | Origem | Significado |
| --- | --- | --- |
| `DXVK_HDR=1 definido antes da inicializacao do DXGI.`, `Exposicao HDR do DXVK desativada pela configuracao.`, `Falha ao solicitar a exposicao HDR ao DXVK.` | `proxy.rs` | início da sessão e resultado de `force_hdr` |
| `Hooks Present, Present1 e ResizeBuffers instalados.` | `hooks.rs` | hooks instalados |
| `Falha ao instalar hooks D3D11 em 60 segundos.` | `hooks.rs` | 120 tentativas, 500 ms entre elas, sem sucesso |
| `Configuracao inicial: ...` | `runtime.rs` | o primeiro `Present` chegou ao plugin |
| `Configuracao recarregada: ...`, `force_hdr so e aplicado na inicializacao do jogo.` | `runtime.rs` | recarga a quente |
| `Display informou pico=... nits e tela cheia=... nits.` ou `Display nao informou luminancia; hdr_peak_nits=auto usa 1000 nits.` | `graphics/mod.rs` | criação do `Renderer` |
| `Pipeline de cor e tonemap inicializado.` | `runtime.rs` | `Renderer` criado para a swap chain atual |
| `Backbuffer LxA formato=N modo=M.` | `graphics/mod.rs` | recursos do quadro (re)criados |
| `Falha ao inicializar o pipeline de cor e tonemap.` | `runtime.rs` | `Renderer::new` falhou: `GetDevice`, compilação do HLSL, sampler ou constant buffer |
| `Falha ao aplicar o passe de cor e tonemap.` | `runtime.rs` | `render` falhou: `GetBuffer(0)`, textura de origem, SRV ou RTV |

`formato=N` é o valor numérico de `DXGI_FORMAT`: 10 é `R16G16B16A16_FLOAT` (scRGB), 24 é `R10G10B10A2_UNORM` (HDR10-PQ), 28 e 29 são `R8G8B8A8_UNORM` e `_SRGB`, 87 e 91 são `B8G8R8A8_UNORM` e `_SRGB`.

O que o log não mostra, e que pode enganar:

- Quadros filtrados (MSAA, menos de 640x480, formato fora da lista) e `enabled=false` não geram mensagem. Com `enabled=false` a sequência para em `Configuracao inicial`. Com o plugin ativo, `Pipeline ... inicializado` sem `Backbuffer` depois indica quadro filtrado.
- Cada falha é registrada uma vez: `error_reported` só é rearmado quando um novo `Renderer` é criado. Erros persistentes ficam silenciosos depois da primeira linha.
- Após `Falha ao inicializar...`, `ensure_renderer` tenta de novo a cada quadro e recompila o HLSL a cada tentativa. Queda forte de FPS junto dessa mensagem é consistente com isso (hipótese lida no código; confirmar comparando com `enabled=false`).
- `Pipeline ... inicializado` repetido indica alternância entre swap chains (`Renderer::matches` compara o ponteiro) ou resets frequentes por `ResizeBuffers`.
- `Falha ao instalar hooks` não prova ausência de hooks: `Present` e `ResizeBuffers` são trocados antes de `Present1`, e uma falha ao obter `IDXGISwapChain1` mantém o laço com os dois primeiros já instalados. A presença de `Configuracao inicial` mostra que o hook de `Present` foi chamado.
- O erro de compilação do HLSL é descartado em `shaders.rs`: só se vê a falha genérica.

### Jogo

- `game.log.txt`, na pasta de usuário do jogo (`Documents\Euro Truck Simulator 2` no Windows; no Proton, o mesmo caminho dentro do prefixo do jogo). A seção "Detected color modes" lista os modos de cor detectados (`INSTALL.txt`).
- Console: `g_console` e `g_developer` iguais a 1 no `config.cfg`, tecla `~`; Ctrl+Shift alterna entre erros e avisos e todas as mensagens; `uset <variável> <valor>` altera variáveis. A wiki não diz se o comando `screenshot` inclui o resultado do passe do plugin; não usá-lo como evidência sem verificar.
- Ao relatar um problema, registrar do `config.cfg`: `r_device`, `r_hdr`, `r_hdr_display_*`, `r_color_*`, `r_gamma`, `r_scale_x` e `r_scale_y`.
- A SCS não documenta os internos do Prism3D. Por que o jogo criou determinado backbuffer, em que ordem compõe a interface ou como trata recursos é observação a registrar, nunca fato presumido.

### Ambiente (Proton e DXVK)

- `DXVK_LOG_LEVEL` (`none`, `error`, `warn`, `info`, `debug`) e `DXVK_LOG_PATH`, que gera `app_dxgi.log`, `app_d3d11.log` e semelhantes.
- `DXVK_HUD` (por exemplo `fps,frametimes,api`) para desempenho.
- `PROTON_LOG=1` grava `steam-$APPID.log` em `PROTON_LOG_DIR` (padrão: home).
- `DXVK_HDR=1` é definido pelo próprio plugin quando `force_hdr=true`; `PROTON_ENABLE_HDR=1` continua necessário (README).

## 1. Analisar o problema

- Identificar o comportamento observado e o esperado.
- Levantar: modo de saída (SDR, HDR10-PQ, scRGB), ambiente (Windows, Proton, GE-Proton, Wayland, Gamescope), versão do plugin e do jogo (o HDR nativo existe desde a 1.57), mods ativos, `photorealism-plugin.cfg` e variáveis do `config.cfg`.
- Determinar quando ocorre: na inicialização, após redimensionar ou alternar tela cheia, só em HDR, só em determinada cena.
- Consultar `AGENTS.md`.
- Não assumir a causa sem evidências.

## 2. Coletar evidências

- Pedir ao usuário, quando o jogo não estiver disponível, o log do plugin, o `game.log.txt`, o `config.cfg` e o log do DXVK ou do Proton, todos da mesma execução.
- Delimitar a sessão no log do plugin e conferir a sequência esperada: sessão, hooks, `Configuracao inicial`, `Pipeline ... inicializado`, `Backbuffer`. O primeiro elo ausente indica onde parar de olhar para frente.
- Verificar o ciclo de vida dos recursos e o estado D3D11 restaurado (`graphics/state.rs`).
- Não modificar o código antes de compreender o comportamento observado.

## 3. Isolar a causa

Alterar uma variável por vez, do menos para o mais invasivo:

1. `enabled=false`: relido em cerca de 1 s, sem reiniciar. Os hooks continuam chamando o original, mas o passe não roda. Separa problema do passe de problema do proxy e dos hooks.
2. Valores neutros (`exposure=0`, `contrast=1`, `saturation=1`, `temperature=6500`, `tint=0`, `highlight_rolloff=0`): o passe roda sem gradação. Isola cópia, decodificação, codificação, corte no pico e estado D3D11 da gradação.
3. Pedir ao usuário que retire `dxgi.dll` de `bin/win_x64/`: exclui o plugin por completo. Outro `dxgi.dll` no mesmo diretório também conflita (README).
4. Variáveis do jogo e do ambiente: `-color_mode sdr` (indicado pela equipe da SCS no fórum) ou HDR desativado no Windows; `hdr_peak_nits` fixo em vez de `auto`. A equipe da SCS diz que o conteúdo é calibrado para HDR por ela, sem previsão de uso com modos automáticos ou adaptativos da tela: desativar o tone mapping dinâmico do monitor antes de atribuir o defeito ao plugin.
5. Mods que alterem clima, iluminação ou materiais: desativar no gerenciador de mods. O plugin recebe o quadro final já com o efeito deles.
6. Overlays e injetores: o código só espera o overlay da Steam estabilizar antes de instalar os hooks; outros hooks em `Present` não têm tratamento equivalente.

Diferenciar o que vem do plugin do que vem do jogo, do driver, do DXVK ou da tela.

## 4. Sintomas e primeiras hipóteses

Hipóteses a confirmar com evidência, não conclusões.

| Sintoma | Onde investigar |
| --- | --- |
| Jogo não abre ou falha ao criar o dispositivo | `CreateDXGIFactory*` devolve `0x80004005` se o DXGI do sistema não carrega (`proxy.rs`); outro `dxgi.dll`; `WINEDLLOVERRIDES="dxgi=n,b"`; `app_dxgi.log` |
| Nenhum log do plugin | DLL não carregada; pasta `photorealism-plugin/` sem escrita; override ausente |
| Log sem `Configuracao inicial` | hook de `Present` nunca chamado: hooks não instalados ou jogo fora de D3D11 (`r_device`) |
| Sem efeito visível | primeiro elo ausente da sequência; quadro filtrado; `enabled=false` |
| Falha ao redimensionar ou alternar tela cheia ou HDR | `ResizeBuffers` exige que nenhuma referência ao backbuffer sobreviva, e a RTV em `FrameResources` é uma. `runtime::reset` usa `try_lock` e não faz nada se o lock estiver ocupado |
| Cores lavadas ou escuras em HDR | `modo=` do log contra a saída real; `hdr_paper_white_nits`; `Display informou pico` contra a especificação do monitor; calibração do jogo (`r_hdr_display_*`); tone mapping da tela |
| Saturação ou balanço de branco diferentes entre SDR e HDR10 | pesos de luminância por modo (regressão já corrigida em `85dc237`); `temperature` e `tint` têm força diferente entre modos (README) |
| Realces duros ou cortados | joelho do `highlight_rolloff` (regressão já corrigida em `256060d`); `tests/tone_curve.rs` |
| Gama dobrado ou banding em SDR | `formato=` no log: a conversão sRGB é manual só nos formatos UNORM (`manual_srgb`) |
| Interface ou geometria quebradas, flicker após o passe | estado D3D11 não restaurado: `PipelineState` cobre só o slot 0 de render target, SRV, sampler e constant buffer, além de shaders, IA, blend, depth, rasterizer e viewports |
| Queda de FPS | comparar com `enabled=false`; laço de falha de `Renderer::new`; custo de `CopyResource` mais o passe em resolução alta; `DXVK_HUD=frametimes` |

## 5. Implementar a correção

- Identificar a causa raiz antes da solução definitiva e evitar soluções que só ocultem o problema.
- Alterações pequenas, localizadas, em branch separada, respeitando SRP e DIP do `AGENTS.md`.
- Tratamento explícito de erros e código idiomático em Rust.
- Não adicionar comentários ao código sem autorização.
- Mudanças no shader, no constant buffer ou no estado D3D11 seguem os contratos da skill `graphics-pipeline`.
- Logs novos seguem o padrão existente: um por evento, português sem acentos, nunca por quadro.
- Lacunas de diagnóstico do projeto (erro de HLSL descartado, log sem data, quadros filtrados sem mensagem, falhas repetidas silenciosas) só devem ser tratadas se o usuário pedir.
- Não modificar componentes sem relação com a correção.

## 6. Validar a correção

- Regra do `AGENTS.md`: escrever antes um teste que reproduza o problema. Regras de domínio (`settings`, `config`, `color`, `PeakNits`) e matemática de shader podem ser reproduzidas no host, com o padrão do espelho em Rust de `tests/tone_curve.rs`.
- Defeitos de estado D3D11, ciclo de vida, hooks e ordem de chamadas não têm reprodução automatizada no host. Registrar isso e validar no jogo comparando a sequência do log e o comportamento visual antes e depois.
- Executar os comandos de validação da skill `graphics-pipeline`. Não usar `tools/build.sh` para validar: ele altera a versão e publica em `dist/`.
- Confirmar que o comportamento original foi corrigido e que a correção não afetou outros modos (SDR, HDR10-PQ, scRGB), a estabilidade nem o desempenho.
- Não considerar o problema resolvido apenas porque o projeto compila.

## 7. Relatar o resultado

Apresentar:

- Problema identificado, condições de reprodução e trechos relevantes dos logs.
- Causa raiz confirmada ou hipóteses ainda não verificadas.
- Arquivos e componentes modificados, com a justificativa técnica.
- Testes executados e onde: host, compilação cruzada ou jogo.
- Limitações conhecidas e riscos remanescentes.

Não afirmar que um problema foi corrigido sem evidências suficientes. Se não for possível testar no jogo, informar explicitamente essa limitação.

## Fontes

- [SCS Modding Wiki: Console](https://modding.scssoft.com/wiki/Documentation/Engine/Console)
- [SCS Modding Wiki: Game user path](https://modding.scssoft.com/wiki/Documentation/Engine/Game_user_path)
- [SCS Modding Wiki: Configuration variables](https://modding.scssoft.com/wiki/Documentation/Engine/Configuration_variables)
- [Fórum SCS: HDR setup](https://forum.scssoft.com/viewtopic.php?t=345222)
- [DXVK: README](https://github.com/doitsujin/dxvk#readme)
- [Proton: README](https://github.com/ValveSoftware/Proton#readme)
