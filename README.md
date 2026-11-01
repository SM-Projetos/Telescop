# Telescop

Aplicativo desktop nativo para pesquisar imagens usando SearXNG e reunir os resultados em uma galeria própria.

> **Status:** em desenvolvimento. O SearXNG é o único mecanismo de busca integrado diretamente ao Telescop. O usuário escolhe a instância SearXNG por meio da variável de ambiente `SEARXNG_INSTANCE_URL`.

## Tecnologias

- **Rust** — lógica da aplicação, integração de rede e processamento dos resultados.
- **Slint** — interface gráfica desktop nativa.
- **Reqwest + Rustls** — requisições HTTP assíncronas com TLS.
- **Serde** — desserialização das respostas JSON.
- **Scraper** — alternativa de leitura da página HTML quando a instância não disponibiliza JSON.
- **SearXNG Search API** — mecanismo de metabusca para a categoria de imagens.

O projeto não utiliza Tauri, Electron, WebView, HTML, CSS ou TypeScript como camada de interface.

## Pesquisa e galeria

O Telescop envia a expressão digitada na barra de pesquisa à categoria `images` da instância SearXNG configurada. A resposta JSON é preferida. Quando a instância desativa esse formato (situação comum em instâncias públicas), o Telescop tenta interpretar a página HTML de resultados.

Os downloads de miniaturas continuam limitados a três operações simultâneas. A galeria apresenta título, instância/provedor, dimensões e informação de origem quando fornecidos pelo SearXNG.

### Configurar a instância SearXNG

O Telescop não fixa uma instância pública nem envia suas pesquisas automaticamente a um servidor escolhido pelo projeto. Configure uma instância pública de sua confiança ou uma instância própria.

No terminal Fish, defina a URL-base da instância antes de iniciar o aplicativo. Informe o endereço principal, sem acrescentar `/search`:

~~~fish
set -x SEARXNG_INSTANCE_URL "https://sua-instancia.example"
cargo run
~~~

A variável também pode apontar para uma instância hospedada em um caminho, por exemplo `https://dominio.example/searxng`. O Telescop acrescenta `/search` automaticamente. URLs com credenciais, parâmetros de consulta ou fragmentos são rejeitadas.

A instância precisa oferecer a categoria de imagens. O Telescop solicita inicialmente `format=json`; se a resposta for HTTP 403 ou não for JSON válido, tenta a página HTML padrão. Se a instância bloquear ambos os formatos, o aplicativo exibirá o erro retornado.

### Privacidade

A pesquisa é enviada ao servidor indicado por `SEARXNG_INSTANCE_URL`. O operador dessa instância pode ter acesso às consultas e aos metadados da requisição. Como o SearXNG é uma metabusca, a instância também encaminha consultas aos mecanismos externos que o administrador habilitou. Escolha um operador em quem confie ou hospede sua própria instância.

O Telescop não escolhe nem troca automaticamente de instância, não seleciona servidores públicos aleatórios e não faz fallback silencioso para outro provedor de busca.

### Licenças e reutilização

Os resultados do SearXNG não oferecem, de maneira uniforme, dados verificáveis de licença para cada imagem. Por isso, o Telescop marca os itens como **Licença não verificada**. A presença de uma imagem na galeria não significa que ela esteja liberada para reutilização, publicação ou uso comercial. Antes de utilizar qualquer arquivo, consulte a página de origem e confirme a licença e as condições de atribuição com o titular dos direitos.

## Desenvolvimento

Requisitos: Rust toolchain com Cargo e dependências nativas exigidas pelo Slint no Linux.

~~~fish
cargo check
cargo test
cargo run
~~~

## Estrutura

- `src/` — código Rust.
- `src/search/` — modelo comum e adaptador SearXNG.
- `ui/` — interface Slint.
- `docs/ARCHITECTURE.md` — arquitetura e etapas planejadas.
- `CHANGELOG.md` — histórico de alterações.

## Licença

MIT. Consulte o arquivo LICENSE.
