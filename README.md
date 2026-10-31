# Telescop

Aplicativo desktop nativo para pesquisar imagens em diferentes fontes e reunir os resultados em uma galeria própria.

> **Status:** em desenvolvimento. A pesquisa padrão usa um adaptador experimental do Startpage; DuckDuckGo está temporariamente desativado no fluxo padrão devido a respostas HTTP 403.

## Tecnologias

- **Rust** — lógica da aplicação, integração com provedores e processamento dos resultados.
- **Slint** — interface gráfica desktop nativa.
- **Reqwest + Rustls** — requisições HTTP assíncronas com TLS.
- **Serde** — desserialização das respostas JSON.

O projeto não utiliza Tauri, Electron, WebView, HTML, CSS ou TypeScript.

## Pesquisa experimental

O primeiro adaptador consulta a API pública de imagens do Openverse. A pesquisa é executada de forma assíncrona para não bloquear a interface e os resultados são convertidos para o modelo comum `ImageResult`.

A integração inicial exibe título, dimensões disponíveis e provedor. A exibição das miniaturas, filtros efetivos e deduplicação serão implementados nas próximas etapas. A disponibilidade e os limites da API dependem do serviço externo.

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
- `src/search/` — modelo comum e adaptadores de busca.
- `ui/` — interface Slint.
- `docs/ARCHITECTURE.md` — arquitetura e etapas planejadas.
- `CHANGELOG.md` — histórico de alterações.

## Fontes em avaliação

Startpage Imagens está ativo experimentalmente, usando parsing de HTML não estável. O adaptador DuckDuckGo permanece no código, mas está desativado no fluxo padrão porque o endpoint interno de imagens retornou HTTP 403. Openverse permanece em espera; Bing, Brave Search e Wikimedia Commons continuam em avaliação. A seleção final dependerá de qualidade, estabilidade, disponibilidade e condições de uso de cada serviço.

## Licença

MIT. Consulte o arquivo [LICENSE](LICENSE).
