# Telescop

Aplicativo desktop nativo para pesquisar imagens em diferentes fontes e reunir os resultados em uma galeria própria.

> **Status:** em desenvolvimento. A pesquisa integrada consulta Wikimedia Commons, Openverse, Europeana e Cleveland Museum of Art; DuckDuckGo Images está disponível como provedor experimental.

## Tecnologias

- **Rust** — lógica da aplicação, integração com provedores e processamento dos resultados.
- **Slint** — interface gráfica desktop nativa.
- **Reqwest + Rustls** — requisições HTTP assíncronas com TLS.
- **Serde** — desserialização das respostas JSON.
- **MediaWiki API, Openverse API, Europeana Search API e Cleveland Museum of Art Open Access API** — fontes estruturadas de imagens.
- **DuckDuckGo Images** — pesquisa experimental por página HTML e endpoint interno de resultados JSON; não é uma API pública documentada.

O projeto não utiliza Tauri, Electron, WebView, HTML, CSS ou TypeScript.

## Pesquisa e galeria

Os provedores selecionados são consultados simultaneamente. Uma falha em uma fonte não impede a apresentação dos resultados obtidos pelas demais. O download das miniaturas continua limitado a três operações simultâneas.

A galeria apresenta o provedor, as dimensões disponíveis e os dados de licença/atribuição recebidos da fonte.

### Fontes integradas

- **Wikimedia Commons:** API oficial do MediaWiki. O adaptador solicita metadados de direitos e mantém somente arquivos identificados como CC0, Marca de Domínio Público, CC BY, CC BY-SA ou Domínio Público.
- **Openverse:** API pública com acesso anônimo gratuito e sujeito a limites de requisição. A consulta permite somente CC0, Domínio Público (PDM), CC BY e CC BY-SA; licenças não comerciais, sem derivações ou não identificadas são excluídas.
- **Europeana:** Search API oficial, filtrada por reusability=open, com verificação adicional para aceitar somente CC0, Marca de Domínio Público, CC BY e CC BY-SA. A chave pessoal de API é gratuita e deve ser configurada por variável de ambiente; não a inclua no código ou em commits. Chaves pessoais são destinadas a uso individual não produtivo. Uma distribuição pública/produção deve solicitar uma chave de projeto conforme as condições oficiais da Europeana.
- **Cleveland Museum of Art:** API Open Access oficial, sem chave. A consulta usa os filtros cc0=1 e has_image=1, e o adaptador confirma novamente o campo share_license_status antes de aceitar o resultado. O conteúdo selecionado é CC0.
- **DuckDuckGo Images (experimental):** o Telescop monta a URL no formato `https://duckduckgo.com/?q=...&iar=images&t=h_`, solicita o HTML, extrai o token de sessão VQD e consulta o endpoint interno `i.js` para obter os registros JSON. Não abre um navegador nem executa JavaScript. Esse endpoint não é uma API pública documentada, pode mudar ou bloquear requisições (por exemplo, HTTP 403). Como os resultados não fornecem metadados confiáveis de licença, aparecem identificados em amarelo como **Licença não verificada** e não devem ser tratados como liberados para reutilização.

### Configuração da Europeana (opcional)

1. Crie uma conta gratuita e solicite uma chave pessoal em https://www.europeana.eu/en/apis.
2. No terminal Fish, configure a variável antes de iniciar o aplicativo:

~~~fish
set -x EUROPEANA_API_KEY "COLE_SUA_CHAVE_AQUI"
cargo run
~~~

Sem essa variável, Wikimedia Commons, Openverse e Cleveland Museum continuam funcionando; a Europeana informa que está sem chave e não interrompe as outras fontes. O DuckDuckGo também pode ser ativado independentemente, mas está sujeito a bloqueios do próprio serviço.

### Política de reutilização

O Telescop aceita somente:

- CC0;
- Marca de Domínio Público (PDM) ou Domínio Público identificado pela fonte;
- CC BY;
- CC BY-SA.

São excluídas licenças com restrição não comercial (NC), sem derivações (ND), direitos reservados, ou registros sem identificação confiável de direitos. CC BY e CC BY-SA exigem atribuição e o cumprimento das condições da licença. A existência de uma API gratuita não significa que todo arquivo do provedor seja livre: os direitos são verificados individualmente para cada resultado. A exceção visual é o DuckDuckGo experimental: seus resultados podem aparecer com licença não verificada, sempre sinalizados e sem declaração de permissão de reutilização.

## Fontes avaliadas, mas fora da pesquisa padrão

- **Startpage:** adaptador experimental baseado em HTML, sem contrato de API pública estável.
- **Yandex Images:** API oficial de imagens tarifada; excluída.
- **Brave Search:** API sujeita a cobrança; excluída.
- **Bing Images:** API oficial aposentada; não há abertura de navegador como alternativa.

O adaptador experimental do DuckDuckGo é a única exceção e consulta um endpoint interno não documentado. O HTML da página é usado somente para obter o token de sessão; os registros de imagens são recebidos em JSON. Os demais provedores usam as APIs e os serviços descritos acima, sem extração de resultados a partir do HTML de suas páginas.

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
