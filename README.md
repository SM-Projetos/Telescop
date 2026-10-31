# Telescop

Aplicativo desktop nativo para pesquisar imagens em diferentes fontes e reunir os resultados em uma galeria própria.

> **Status:** em desenvolvimento. A pesquisa integrada utiliza a API documentada do Wikimedia Commons.

## Tecnologias

- **Rust** — lógica da aplicação, integração com provedores e processamento dos resultados.
- **Slint** — interface gráfica desktop nativa.
- **Reqwest + Rustls** — requisições HTTP assíncronas com TLS.
- **Serde** — desserialização das respostas JSON.
- **MediaWiki API** — pesquisa integrada de imagens no Wikimedia Commons.

O projeto não utiliza Tauri, Electron, WebView, HTML, CSS ou TypeScript.

## Pesquisa e galeria

A pesquisa integrada utiliza a API documentada do Wikimedia Commons para localizar arquivos e solicitar miniaturas. Os resultados são convertidos para o modelo comum `ImageResult` e as miniaturas são baixadas em paralelo, com limite de três requisições simultâneas.

Falhas individuais de download ou decodificação não interrompem toda a pesquisa. O status apresenta a quantidade de resultados, miniaturas carregadas e exemplos de falhas para facilitar o diagnóstico.

## Fontes avaliadas

- **Wikimedia Commons:** fonte integrada, através da API oficial do MediaWiki.
- **Yandex Images:** possui uma API oficial, mas a pesquisa de imagens é tarifada. A documentação oficial informa cobrança de 915 rublos por mil consultas; por isso, não será integrada ao Telescop enquanto o projeto mantiver o requisito de usar apenas serviços gratuitos.
- **DuckDuckGo e Startpage:** adaptadores experimentais preservados no código, mas desativados no fluxo padrão. O endpoint interno do DuckDuckGo retornou HTTP 403, e o parser HTML do Startpage depende de uma estrutura que pode mudar.
- **Openverse:** adaptador preservado no código, mas fora do fluxo padrão.

Não são usados scraping de páginas de resultados nem mecanismos que exijam assinatura, créditos pagos ou chave com cobrança.

O Wikimedia Commons contém conteúdo sob diferentes licenças. Verifique a licença e as condições de cada arquivo antes de reutilizá-lo.

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
