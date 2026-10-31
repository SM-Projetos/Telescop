# Telescop

Aplicativo desktop nativo para pesquisar imagens em diferentes fontes e reunir os resultados em uma galeria própria.

> **Status:** em desenvolvimento. A pesquisa padrão utiliza a API documentada do Wikimedia Commons. DuckDuckGo e Startpage permanecem como adaptadores experimentais e não participam do fluxo padrão.

## Tecnologias

- **Rust** — lógica da aplicação, integração com provedores e processamento dos resultados.
- **Slint** — interface gráfica desktop nativa.
- **Reqwest + Rustls** — requisições HTTP assíncronas com TLS.
- **Serde** — desserialização das respostas JSON.
- **MediaWiki API** — pesquisa de arquivos no Wikimedia Commons.

O projeto não utiliza Tauri, Electron, WebView, HTML, CSS ou TypeScript.

## Pesquisa e galeria

A pesquisa padrão utiliza a API do Wikimedia Commons para localizar arquivos e solicitar miniaturas. Os resultados são convertidos para o modelo comum `ImageResult` e as miniaturas são baixadas em paralelo, com limite de oito requisições simultâneas.

Falhas individuais de download ou decodificação não interrompem toda a pesquisa. O status apresenta a quantidade de resultados, miniaturas carregadas e exemplos de falhas para facilitar o diagnóstico.

A disponibilidade da API e dos arquivos depende do serviço externo. O Wikimedia Commons contém apenas conteúdo disponibilizado sob suas políticas de licenciamento; verifique a licença e as condições de cada arquivo antes de reutilizá-lo.

## Adaptadores experimentais

- **DuckDuckGo Imagens:** mantido no código, mas desativado no fluxo padrão porque o endpoint interno utilizado retornou HTTP 403.
- **Startpage Imagens:** mantido no código; depende da estrutura HTML do site, que não é uma API pública estável.
- **Openverse:** mantido no código, mas fora do fluxo padrão.

Esses adaptadores poderão ser reavaliados quando houver uma integração confiável e compatível com as condições dos respectivos serviços.

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

## Licença

MIT. Consulte o arquivo [LICENSE](LICENSE).
