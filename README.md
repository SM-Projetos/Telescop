# Telescop

Aplicativo desktop nativo para pesquisar imagens em diferentes fontes e reunir os resultados em uma galeria própria.

> **Status:** em desenvolvimento. A pesquisa integrada consulta Wikimedia Commons, Openverse, Europeana e Cleveland Museum of Art.

## Tecnologias

- **Rust** — lógica da aplicação, integração com provedores e processamento dos resultados.
- **Slint** — interface gráfica desktop nativa.
- **Reqwest + Rustls** — requisições HTTP assíncronas com TLS.
- **Serde** — desserialização das respostas JSON.
- **MediaWiki API, Openverse API, Europeana Search API e Cleveland Museum of Art Open Access API** — fontes de imagens.

O projeto não utiliza Tauri, Electron, WebView, HTML, CSS ou TypeScript.

## Pesquisa e galeria

As quatro fontes são consultadas simultaneamente. Uma falha em uma API não impede a apresentação dos resultados obtidos pelas demais. O download das miniaturas continua limitado a três operações simultâneas.

A galeria apresenta o provedor, as dimensões disponíveis e os dados de licença/atribuição recebidos da fonte.

### Fontes integradas

- **Wikimedia Commons:** API oficial do MediaWiki. O adaptador solicita metadados de direitos e mantém somente arquivos identificados como CC0, Marca de Domínio Público, CC BY, CC BY-SA ou Domínio Público.
- **Openverse:** API pública com acesso anônimo gratuito e sujeito a limites de requisição. A consulta permite somente CC0, Domínio Público (PDM), CC BY e CC BY-SA; licenças não comerciais, sem derivações ou não identificadas são excluídas.
- **Europeana:** Search API oficial, filtrada por reusability=open, com verificação adicional para aceitar somente CC0, Marca de Domínio Público, CC BY e CC BY-SA. A chave pessoal de API é gratuita e deve ser configurada por variável de ambiente; não a inclua no código ou em commits. Chaves pessoais são destinadas a uso individual não produtivo. Uma distribuição pública/produção deve solicitar uma chave de projeto conforme as condições oficiais da Europeana.
- **Cleveland Museum of Art:** API Open Access oficial, sem chave. A consulta usa os filtros cc0=1 e has_image=1, e o adaptador confirma novamente o campo share_license_status antes de aceitar o resultado. O conteúdo selecionado é CC0.

### Configuração da Europeana (opcional)

1. Crie uma conta gratuita e solicite uma chave pessoal em https://www.europeana.eu/en/apis.
2. No terminal Fish, configure a variável antes de iniciar o aplicativo:

~~~fish
set -x EUROPEANA_API_KEY "COLE_SUA_CHAVE_AQUI"
cargo run
~~~

Sem essa variável, Wikimedia Commons, Openverse e Cleveland Museum continuam funcionando; a Europeana informa que está sem chave e não interrompe as outras fontes.

### Política de reutilização

O Telescop aceita somente:

- CC0;
- Marca de Domínio Público (PDM) ou Domínio Público identificado pela fonte;
- CC BY;
- CC BY-SA.

São excluídas licenças com restrição não comercial (NC), sem derivações (ND), direitos reservados, ou registros sem identificação confiável de direitos. CC BY e CC BY-SA exigem atribuição e o cumprimento das condições da licença. A existência de uma API gratuita não significa que todo arquivo do provedor seja livre: os direitos são verificados individualmente para cada resultado.

## Fontes avaliadas, mas fora da pesquisa padrão

- **DuckDuckGo:** endpoint interno/indocumentado retornou HTTP 403.
- **Startpage:** adaptador experimental baseado em HTML, sem contrato de API pública estável.
- **Yandex Images:** API oficial de imagens tarifada; excluída.
- **Brave Search:** API sujeita a cobrança; excluída.
- **Bing Images:** API oficial aposentada; não há abertura de navegador como alternativa.

Não são usados scraping de páginas de resultados nem mecanismos que exijam assinatura, créditos pagos ou chave com cobrança.

## Desenvolvimento

Requisitos: Rust toolchain com Cargo e dependências nativas exigidas pelo Slint no Linux.

~~~fish
cargo check
cargo run
~~~

## Estrutura

- src/ — código Rust.
- src/search/ — modelo comum e adaptadores de busca.
- ui/ — interface Slint.
- docs/ARCHITECTURE.md — arquitetura e etapas planejadas.
- CHANGELOG.md — histórico de alterações.

## Licença

MIT. Consulte o arquivo LICENSE.
