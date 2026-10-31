# Telescop — Arquitetura inicial

## Objetivo

Aplicativo desktop nativo para pesquisar imagens em diferentes fontes e reunir os resultados em uma galeria própria.

## Tecnologias

- Rust para lógica, rede, normalização e gerenciamento de estado.
- Slint para a interface desktop nativa.
- Operações de rede assíncronas, sem bloquear a thread da interface.

## Organização

- `src/main.rs`: inicialização da aplicação e integração inicial com a UI.
- `src/lib.rs`: módulos da biblioteca da aplicação.
- `src/search/`: modelos, coordenação de buscas e adaptadores de provedores.
- `src/privacy/`: políticas de conexão, proxy e Tor (etapa futura).
- `ui/`: componentes e telas Slint.
- `docs/`: decisões e documentação técnica.

## Modelo de resultado

Todos os provedores deverão produzir um `ImageResult` comum, contendo URL da imagem, miniatura opcional, título, página de origem, provedor e dimensões conhecidas.

## Adaptadores de busca

Cada fonte será isolada em um adaptador independente. O núcleo deverá coordenar consultas, receber resultados progressivamente, normalizar dados e remover duplicatas.

Wikimedia Commons é o provedor de pesquisa integrado e utiliza a API documentada do MediaWiki para pesquisar arquivos e solicitar URLs de miniaturas. Os resultados são normalizados para `ImageResult`; os downloads das miniaturas ocorrem em paralelo, com limite de três operações simultâneas. Falhas de rede, status HTTP e formatos não suportados são informados no status da aplicação. Yandex Images possui uma API oficial, mas a pesquisa de imagens é tarifada (a tabela consultada lista aproximadamente US$ 7,50 por mil consultas para clientes faturados em dólar) e está fora do escopo gratuito do projeto. O acordo de uso do Yandex também reserva à empresa o direito de restringir requisições automatizadas; por isso, não será usado scraping HTML para contornar a API. DuckDuckGo e Startpage permanecem como adaptadores experimentais fora do fluxo padrão: o endpoint interno do DuckDuckGo retornou HTTP 403, e o parser HTML do Startpage depende de uma estrutura que pode mudar. Openverse permanece implementado, mas fora do fluxo padrão até resolver a autenticação. Não presumir que scraping seja permitido ou sustentável.

## Privacidade

Os modos Normal, Anônimo, Tor e Proxy serão implementados com comportamento explícito. Falhas de Tor ou proxy não poderão provocar fallback silencioso para conexão direta. A aplicação não deverá prometer anonimato absoluto.

## Etapas

1. Fundação Rust + Slint e identidade visual.
2. Contratos dos provedores e motor assíncrono.
3. Primeiro adaptador experimental.
4. Galeria, deduplicação, dimensões e filtros.
5. Fontes adicionais e recursos de privacidade.
6. Testes, documentação e empacotamento.
