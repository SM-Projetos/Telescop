# Telescop

Aplicativo desktop nativo para pesquisar imagens em diferentes fontes e reunir os resultados em uma galeria própria.

> **Status:** fundação inicial em desenvolvimento. A interface e a estrutura Rust + Slint foram iniciadas; os mecanismos de busca ainda serão implementados e avaliados.

## Tecnologias

- **Rust** — lógica da aplicação, integração com provedores e processamento dos resultados.
- **Slint** — interface gráfica desktop nativa.

O projeto não utiliza Tauri, Electron, WebView, HTML, CSS ou TypeScript.

## Objetivos iniciais

- Pesquisar em múltiplas fontes independentes.
- Exibir resultados progressivamente em uma galeria própria.
- Normalizar resultados e identificar imagens duplicadas.
- Oferecer filtros por tamanho, orientação e proporção.
- Ter controles claros de privacidade, proxy e Tor, sem fallback silencioso para conexão direta.

## Desenvolvimento

Requisitos: Rust toolchain com Cargo.

```bash
cargo run
```

Para validar a compilação:

```bash
cargo check
```

## Estrutura

- `src/` — código Rust.
- `src/search/` — modelo e futura coordenação de buscas.
- `ui/` — interface Slint.
- `docs/ARCHITECTURE.md` — arquitetura e etapas planejadas.
- `CHANGELOG.md` — histórico de alterações.

## Fontes em avaliação

Bing, DuckDuckGo, Brave Search, Startpage, Openverse e Wikimedia Commons são candidatas iniciais. A seleção final dependerá de testes de qualidade, estabilidade, disponibilidade e condições de uso de cada serviço.

## Licença

MIT. Consulte o arquivo [LICENSE](LICENSE).
