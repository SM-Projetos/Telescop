# Telescop

Aplicativo desktop nativo para Linux que incorpora o mecanismo de navegação Servo à interface Slint. Nesta etapa experimental, o Telescop abre a página real do DuckDuckGo Images dentro da própria janela.

> **Status:** protótipo experimental. A pesquisa usa a página web do DuckDuckGo; o Telescop não consulta API de imagens nem extrai resultados para uma galeria própria.

## Experiência atual

- Barra de pesquisa na parte superior.
- Pesquisa direcionada à seção de imagens do DuckDuckGo, em português do Brasil.
- Página real renderizada dentro da janela do Telescop.
- Navegação por cliques, rolagem e teclado na área web.
- Controles para voltar, avançar e recarregar.
- Sem galeria própria, raspagem de resultados ou download automático de imagens.

O texto digitado é codificado como parâmetro de URL e enviado ao endereço do DuckDuckGo com os parâmetros `ia=images`, `iax=images` e `kl=br-pt`. A página é carregada diretamente pelo Servo. A aplicação não utiliza a API de pesquisa nem o endpoint interno de imagens do DuckDuckGo.

## Tecnologias

- **Rust** — coordenação da aplicação.
- **Slint** — interface desktop nativa.
- **Servo 0.7** — mecanismo de navegação e renderização web incorporado.
- **WGPU 3.0 + Surfman** — integração gráfica entre Servo e Slint.
- **smol** — sinalização assíncrona do ciclo de eventos do Servo.

A integração é baseada no exemplo oficial [Slint + Servo](https://github.com/slint-ui/slint/tree/master/examples/servo). O Slint está fixado na revisão `5fa335de646b0b0ef22f7dc500810da78354e6df`, usada pelo exemplo para manter alinhamento com a integração WGPU 3.0 e suas dependências.

## Compilação no Linux

A compilação do Servo é significativamente mais pesada que a versão anterior do Telescop. Além de Rust/Cargo, o ambiente pode precisar de ferramentas C/C++, Clang/LLVM, CMake, Python, bibliotecas de desenvolvimento de fontes, X11/Wayland, OpenGL/Vulkan e componentes de sistema usados pelo Servo.

Consulte os pré-requisitos atualizados do Servo antes de compilar:

- [Servo — instalação e compilação](https://book.servo.org/)
- [Slint — requisitos de compilação no Linux](https://docs.slint.dev/latest/docs/slint/guide/platforms/)

Depois de instalar as dependências do sistema:

~~~fish
cargo check
cargo test
cargo run
~~~

A primeira compilação baixa e compila uma quantidade grande de dependências do Servo e pode consumir bastante memória, espaço em disco e tempo.

## Limitações conhecidas

- A integração Servo + Slint é experimental e não deve ser considerada pronta para produção.
- A compatibilidade da página do DuckDuckGo depende do suporte web implementado pelo Servo; algumas funcionalidades podem diferir de Firefox/Chromium.
- Não há abas, histórico persistente, favoritos, downloads, gerenciamento de cookies pela interface ou painel de desenvolvedor.
- Não há extração de imagens, galeria própria ou verificação automática de licenças nesta etapa.
- O navegador usa o armazenamento e a configuração padrão disponibilizados pelo Servo; o Telescop ainda não oferece controles de privacidade próprios.

## Estrutura

- `src/main.rs` — inicialização do Slint, criação do Servo e composição do endereço de pesquisa.
- `src/webview/` — integração do Servo, eventos de mouse/teclado/rolagem e renderização.
- `ui/main.slint` — barra de pesquisa e área de navegação.
- `build.rs` — geração dos bindings OpenGL e compilação da interface.
- `docs/ARCHITECTURE.md` — arquitetura e limitações técnicas.
- `CHANGELOG.md` — histórico de alterações.

## Licença

O Telescop permanece sob MIT. A integração incorpora código do exemplo oficial Slint + Servo, mantendo os avisos de copyright e licença dos arquivos de origem. O arquivo `src/webview/rendering_context/surfman_context.rs` mantém o cabeçalho Mozilla Public License 2.0; a licença está incluída em `LICENSES/MPL-2.0.txt`.
