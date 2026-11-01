# Telescop — Arquitetura inicial

## Objetivo

Aplicativo desktop nativo para pesquisar imagens usando uma instância SearXNG e reunir os resultados em uma galeria própria.

## Tecnologias

- Rust para lógica, rede, normalização e gerenciamento de estado.
- Slint para a interface desktop nativa.
- Reqwest para operações HTTP assíncronas, sem bloquear a thread da interface.
- Serde para desserialização JSON.
- Scraper para interpretar a página HTML de resultados quando a instância não habilita JSON.

## Organização

- `src/main.rs`: inicialização, coordenação da busca SearXNG, download de miniaturas e integração com a UI.
- `src/lib.rs`: módulos da biblioteca da aplicação.
- `src/search/mod.rs`: exporta o adaptador único de pesquisa.
- `src/search/searxng.rs`: comunicação com a instância SearXNG, normalização JSON e alternativa HTML.
- `src/search/model.rs`: modelo comum com URLs, dimensões, provedor e metadados disponíveis.
- `ui/main.slint`: interface e galeria.

## Fluxo de pesquisa

1. O usuário informa a consulta na interface.
2. O Telescop lê `SEARXNG_INSTANCE_URL`, configurada no ambiente antes da inicialização.
3. A consulta é enviada ao endpoint `/search` da instância, com `categories=images`, idioma `pt-BR` e pesquisa segura moderada.
4. O aplicativo solicita `format=json`. Se a instância responder HTTP 403 ou entregar conteúdo que não seja JSON válido, tenta novamente sem o parâmetro de formato e interpreta os cartões HTML de imagem.
5. Os resultados são normalizados em `ImageResult`.
6. As miniaturas são baixadas e decodificadas em operações assíncronas, com concorrência limitada a três.
7. A galeria exibe as imagens, títulos, dimensões e dados de origem.

## Modelo de resultado

O modelo `ImageResult` contém URL da imagem, miniatura opcional, título, página de origem, provedor, dimensões, autor, licença, URL da licença e atribuição. Os campos de licença permanecem vazios quando a instância não fornece dados confiáveis.

## Licenças e atribuição

SearXNG agrega imagens de mecanismos e fontes diferentes. Os resultados não oferecem dados uniformes e verificáveis de licença. Portanto, o adaptador não presume que uma imagem é livre, não converte a origem em uma licença e deixa os campos de licença vazios. A UI apresenta esses resultados como **Licença não verificada**. O usuário deve abrir a página de origem e confirmar as condições de uso antes de reutilizar o arquivo.

## Falhas e downloads

Falhas HTTP, JSON inválido, bloqueio de HTML ou ausência de resultados de imagem são reportados no status da aplicação. Os downloads de miniaturas são limitados a três operações simultâneas. Erros HTTP e de decodificação de cada imagem não impedem a apresentação das demais imagens válidas.

## Privacidade

A instância indicada em `SEARXNG_INSTANCE_URL` recebe a consulta. O operador pode registrar a pesquisa e os metadados da requisição, e a instância pode encaminhar a consulta aos mecanismos externos que seu administrador configurou. O Telescop não seleciona uma instância pública por conta própria nem alterna silenciosamente entre provedores.

Os modos Normal, Anônimo e Tor exibidos na interface ainda não estão implementados funcionalmente. Não devem ser interpretados como garantias de anonimato.

## Etapas

1. Fundação Rust + Slint e identidade visual.
2. Integração exclusiva com SearXNG, resposta JSON e alternativa HTML.
3. Modelo comum de imagem e galeria com miniaturas.
4. Deduplicação, filtros, visualização integral, download e favoritos.
5. Recursos de privacidade com garantias explícitas.
6. Testes, documentação e empacotamento.
