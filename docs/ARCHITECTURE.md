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

DuckDuckGo Imagens e Startpage Imagens possuem adaptadores experimentais baseados em consulta HTTP às páginas/endpoints dos buscadores. O endpoint de imagens do DuckDuckGo exige token de sessão VQD; o parser HTML do Startpage é sujeito a alterações. Openverse permanece implementado, mas fora do fluxo padrão até resolver a autenticação. Bing, Brave Search e Wikimedia Commons seguem em avaliação. A inclusão definitiva dependerá de testes de qualidade, estabilidade, disponibilidade e condições de uso. Não presumir que scraping seja permitido ou sustentável.

## Privacidade

Os modos Normal, Anônimo, Tor e Proxy serão implementados com comportamento explícito. Falhas de Tor ou proxy não poderão provocar fallback silencioso para conexão direta. A aplicação não deverá prometer anonimato absoluto.

## Etapas

1. Fundação Rust + Slint e identidade visual.
2. Contratos dos provedores e motor assíncrono.
3. Primeiro adaptador experimental.
4. Galeria, deduplicação, dimensões e filtros.
5. Fontes adicionais e recursos de privacidade.
6. Testes, documentação e empacotamento.
