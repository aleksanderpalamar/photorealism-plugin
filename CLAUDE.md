# AGENT.md

Você deve respeitar as regras deste projeto em todas as alterações realizadas.
Todo código produzido deve permanecer em conformidade com estas regras.

## Princípios de arquitetura

- Aplicar o Princípio da Responsabilidade Única (SRP).
  Cada módulo, struct, função ou componente deve possuir uma responsabilidade clara e bem definida.

- Aplicar o Princípio da Inversão de Dependência (DIP).
  Módulos de alto nível não devem depender diretamente de módulos de baixo nível.
  Ambos devem depender de abstrações.

## Organização do código

- Seguir princípios de Clean Code.

- Cada arquivo de código-fonte deve possuir, preferencialmente, no máximo 200 linhas.
  Caso ultrapasse esse limite, avaliar a divisão do arquivo em módulos menores e coesos.

- Para cada alteração, ajustes, correções de bugs e etc, deve ser realizados em branches separadas.

-  Não adicionar comentarios no código sem a minha permissão.

- Funções devem ser pequenas e possuir uma única responsabilidade.

- Evitar condicionais profundamente aninhadas.

- Preferir:
  - early return;
  - `match`;
  - `if let`;
  - `let else`;
  - decomposição em funções menores;
  - tipos e enums para representar estados explicitamente.

- Evitar duplicação de código.
  Quando houver comportamento realmente compartilhado, extrair uma abstração apropriada.

- Não criar abstrações prematuramente.
  Uma abstração deve existir porque resolve um problema real de design,
  e não apenas para antecipar uma possível necessidade futura.

## Códigos de conduta

Você pode ser proativo dentro do contexto da tarefa, mas nunca fora dele. Se, durante uma tarefa, você perceber que um trecho de código precisa de refatoração ou correção que não foi solicitada, não a execute. Apenas registre o que foi identificado no relatório final, para que eu decida se entra em uma próxima tarefa.

## Skills

- Utilizar sempre a skill graphics-pipeline quando for mexer na pipeline grafica do plugin, como implementação de features novas e refatoração.
- Utilizar sempre a skill graphics-debugging quando surgir correção de crashes, artefatos visuais, cores erradas, ausência de efeito, falhas de inicialização e regressões de desempenho no photorealism-plugin.
- Quando a(s) tarefa(s) envolverem tanto implementação quanto investigação de problemas, use as duas skills `graphics-pipeline` e `graphics-debugging` de forma complementar.

## Rust

- Priorizar código idiomático em Rust.

- Utilizar o sistema de tipos para representar regras e estados sempre que possível.

- Preferir enums em vez de flags booleanas quando existirem múltiplos estados possíveis.
Evitar operações que assumem sucesso sem verificar erros.
Em Option<T>, evitar unwrap() e expect() quando a ausência for um caso normal sem antes tratar isso.
Evitar panic!, unreachable! e assert! para situações esperadas de execução.
Preferir tratamento explícito do erro.

- Utilizar Option<T> quando um valor pode não existir.

- Não silenciar warnings sem uma justificativa clara.

## Testes

- Todo comportamento relevante deve possuir testes.

- Algoritmos e regras de domínio devem ser testáveis.

- Correções de bugs devem, sempre que possível, incluir um teste que reproduza o problema antes da correção.

- Funções determinísticas de domínio devem possuir testes unitários.

## Qualidade

Antes de considerar uma alteração concluída:

1. O projeto deve compilar sem erros.
2. Novos warnings não devem ser introduzidos sem justificativa.
3. Os testes existentes devem continuar passando.
4. Novos comportamentos devem possuir testes quando aplicável.
5. O código deve permanecer simples, legível e consistente com a arquitetura existente.