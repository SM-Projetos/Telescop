# Telescop

Aplicativo desktop nativo para pesquisar imagens em diferentes fontes e reunir os resultados em uma galeria própria.

> **Status:** em desenvolvimento. A pesquisa integrada utiliza a API documentada do Wikimedia Commons. O Bing Imagens pode ser aberto no navegador com a consulta atual.

## Tecnologias

- **Rust** — lógica da aplicação, integração com provedores e processamento dos resultados.
- **Slint** — interface gráfica desktop nativa.
- **Reqwest + Rustls** — requisições HTTP assíncronas com TLS.
- **Serde** — desserialização das respostas JSON.
- **MediaWiki API** — pesquisa integrada de imagens no Wikimedia Commons.
- **webbrowser** — abertura de pesquisas externas no navegador padrão.

O projeto não utiliza Tauri, Electron, WebView, HTML, CSS ou TypeScript.

## Pesquisa e galeria

A pesquisa integrada utiliza a API documentada do Wikimedia Commons para localizar arquivos e solicitar miniaturas. Os resultados são convertidos para o modelo comum `ImageResult` e as miniaturas são baixadas em paralelo, com limite de três requisições simultâneas.

Falhas individuais de download ou decodificação não interrompem toda a pesquisa. O status apresenta a quantidade de resultados, miniaturas carregadas e exemplos de falhas para facilitar o diagnóstico.

## Bing Imagens

A Microsoft encerrou as Bing Search APIs, incluindo a API de pesquisa de imagens, em 11 de agosto de 2025. O produto foi desativado e não aceita novos cadastros. O recurso **Abrir Bing Imagens** abre a pesquisa no navegador padrão; ele não tenta extrair os resultados da página nem inseri-los na galeria.

Não usamos scraping do Bing nem serviços que exijam assinatura ou chave paga para essa função.

## Fontes e condições de uso

- **Wikimedia Commons:** fonte integrada, através da API oficial do MediaWiki.
- **Bing Imagens:** pesquisa externa aberta no navegador.
- **DuckDuckGo e Startpage:** adaptadores experimentais preservados no código, mas desativados no fluxo padrão. O endpoint interno do DuckDuckGo retornou HTTP 403, e o parser HTML do Startpage depende de uma estrutura que pode mudar.
- **Openverse:** adaptador preservado no código, mas fora do fluxo padrão.

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
