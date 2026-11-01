# Telescop — Arquitetura inicial

## Objetivo

Aplicativo desktop nativo para pesquisar imagens em diferentes fontes e reunir os resultados em uma galeria própria.

## Tecnologias

- Rust para lógica, rede, normalização e gerenciamento de estado.
- Slint para a interface desktop nativa.
- Operações de rede assíncronas, sem bloquear a thread da interface.

## Organização

- src/main.rs: inicialização da aplicação, coordenação das buscas e integração com a UI.
- src/lib.rs: módulos da biblioteca da aplicação.
- src/search/: modelo comum e adaptadores independentes por provedor.
- src/search/model.rs: URLs, dimensões, provedor e metadados de direitos/atribuição.
- ui/: componentes e telas Slint.
- docs/: decisões e documentação técnica.

## Modelo de resultado

Todos os provedores produzem um ImageResult comum, contendo URL da imagem, miniatura opcional, título, página de origem, provedor, dimensões, autor, licença, URL da licença e atribuição.

## Provedores integrados

As buscas dos provedores selecionados — Wikimedia Commons, Openverse, Europeana, Cleveland Museum of Art e DuckDuckGo — são executadas em paralelo. A falha de uma fonte não cancela as demais. O núcleo reúne os resultados e processa miniaturas com concorrência limitada a três downloads.

### Política de direitos

Os provedores com metadados de direitos confiáveis só aceitam direitos identificados como:

- CC0;
- Public Domain Mark (PDM) ou Domínio Público identificado pela fonte;
- CC BY;
- CC BY-SA.

Licenças NC (uso não comercial), ND (sem derivações), direitos reservados e registros sem identificação confiável são excluídos.

O adaptador do Wikimedia Commons solicita extmetadata e verifica LicenseUrl/LicenseShortName. O Openverse usa uma lista explícita de licenças permitidas na consulta e valida a licença retornada. A Europeana filtra por reusability=open e ainda valida a URL de direitos do registro, aceitando apenas CC0, PDM, CC BY e CC BY-SA. O Cleveland Museum consulta somente cc0=1 e confirma share_license_status=CC0 antes de criar o resultado.

O DuckDuckGo é uma exceção experimental: a página HTML é solicitada com os parâmetros q, iar=images e t=h_; o adaptador extrai o token VQD e consulta o endpoint interno i.js para receber JSON. Não há API pública documentada nem metadados de licença confiáveis. Esses resultados são apresentados com o rótulo visual amarelo "Licença não verificada" e não são considerados liberados para reutilização.

A API da Europeana exige chave gratuita. O aplicativo lê a chave pessoal a partir de EUROPEANA_API_KEY, sem armazená-la no repositório. Chaves pessoais destinam-se a uso individual não produtivo; uma distribuição pública em produção deve utilizar chave de projeto concedida para esse fim.

## Falhas e downloads

Falhas de rede ou respostas inválidas de um provedor são registradas sem impedir resultados das outras fontes. Os downloads de miniaturas são limitados a três operações simultâneas. Erros HTTP e de decodificação são mostrados no status da aplicação.

## Privacidade

Os modos Normal, Anônimo, Tor e Proxy ainda não estão implementados funcionalmente. Falhas de Tor ou proxy não poderão provocar fallback silencioso para conexão direta. A aplicação não deverá prometer anonimato absoluto.

## Etapas

1. Fundação Rust + Slint e identidade visual.
2. Adaptadores Wikimedia Commons, Openverse, Europeana e Cleveland Museum of Art; integração experimental do DuckDuckGo.
3. Modelo comum com metadados de direitos e atribuição.
4. Galeria, deduplicação, filtros e carregamento progressivo.
5. Fontes adicionais e recursos de privacidade.
6. Testes, documentação e empacotamento.
