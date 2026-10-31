# Telescop

Aplicativo desktop nativo para pesquisar imagens em diferentes fontes e reunir os resultados em uma galeria própria.

> **Status:** em desenvolvimento. A pesquisa utiliza a API oficial do Brave Search quando uma chave está configurada e recorre ao Wikimedia Commons como alternativa.

## Tecnologias

- **Rust** — lógica da aplicação, integração com provedores e processamento dos resultados.
- **Slint** — interface gráfica desktop nativa.
- **Reqwest + Rustls** — requisições HTTP assíncronas com TLS.
- **Serde** — desserialização das respostas JSON.
- **Brave Search API** — pesquisa web de imagens.
- **MediaWiki API** — fonte alternativa sem chave de API.

O projeto não utiliza Tauri, Electron, WebView, HTML, CSS ou TypeScript.

## Pesquisa e galeria

Quando `BRAVE_SEARCH_API_KEY` está definida, a pesquisa usa o endpoint oficial de imagens do Brave, com país Brasil, idioma português e SafeSearch rigoroso. As miniaturas e os URLs originais são convertidos para o modelo comum `ImageResult`.

Se a chave não estiver configurada, se a API falhar ou se o Brave não retornar resultados, o aplicativo utiliza o Wikimedia Commons como alternativa. O carregamento das miniaturas ocorre em paralelo, com limite de três requisições simultâneas.

Falhas individuais de download ou decodificação não interrompem toda a pesquisa. O status apresenta a quantidade de resultados, miniaturas carregadas e exemplos de falhas para facilitar o diagnóstico.

### Configurar a chave do Brave

1. Crie uma conta e ative o plano Search no [painel oficial da API Brave](https://api-dashboard.search.brave.com/).
2. Gere uma chave de API no painel.
3. Defina a variável de ambiente antes de iniciar o aplicativo.

No Fish, leia a chave sem exibi-la nem colocá-la diretamente no comando:

```fish
read --silent --prompt-str="Chave da API Brave: " BRAVE_SEARCH_API_KEY
echo
set -x BRAVE_SEARCH_API_KEY $BRAVE_SEARCH_API_KEY
cargo run
```

No Bash:

```bash
read -s -p "Chave da API Brave: " BRAVE_SEARCH_API_KEY
printf '\n'
export BRAVE_SEARCH_API_KEY
cargo run
```

A chave fica no ambiente do processo e não deve ser colocada no código, em arquivos versionados ou em mensagens públicas. Sem a variável, o Telescop continua funcionando com o Wikimedia Commons.

A disponibilidade, as cotas e os custos da API dependem do plano e das condições vigentes no painel do Brave.

O Wikimedia Commons contém apenas conteúdo disponibilizado sob suas políticas de licenciamento; verifique a licença e as condições de cada arquivo antes de reutilizá-lo. A API do Brave não concede direitos sobre imagens de terceiros; respeite os termos dos sites de origem.

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
