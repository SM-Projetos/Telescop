# Telescop — Arquitetura do protótipo com navegador incorporado

## Objetivo

Aplicativo desktop nativo para Linux que mantém a interface Slint e renderiza uma página web real dentro da janela usando o mecanismo Servo. A primeira página é o DuckDuckGo Images.

## Tecnologias e versões

- Rust edition 2024.
- Slint com WGPU 3.0, fixado na revisão `5fa335de646b0b0ef22f7dc500810da78354e6df` usada pelo exemplo oficial de integração.
- Servo 0.7.
- Surfman para o contexto de renderização OpenGL usado pelo Servo.
- WGPU para transportar a imagem renderizada para a superfície Slint.
- smol para sinalizar o ciclo de eventos do Servo.

## Organização

- `src/main.rs`: seleciona o backend WGPU, cria a janela, inicializa Servo após a criação do dispositivo gráfico e converte as pesquisas em URLs DuckDuckGo.
- `src/lib.rs`: compila os componentes Slint e exporta o módulo de navegação.
- `src/webview/`: ponte de renderização, delegado Servo, sinalização do event loop e conversão dos eventos de entrada.
- `src/webview/webview.slint`: componente visual que exibe o frame do navegador e encaminha eventos de entrada.
- `src/webview/rendering_context/`: implementação de renderização Surfman, compartilhamento/transferência de textura Vulkan e fallback de renderização por software.
- `ui/main.slint`: identidade visual, campo de pesquisa, controles de navegação e área onde a página web é exibida.
- `build.rs`: gera bindings OpenGL usados pelo caminho Vulkan e compila os arquivos Slint.

## Fluxo de pesquisa

1. O usuário digita uma consulta e pressiona Enter ou clica em Pesquisar.
2. Rust codifica a expressão com o serializador de formulários URL, preservando acentos e caracteres especiais.
3. O endereço construído é `https://duckduckgo.com/?q=...&ia=images&iax=images&kl=br-pt`.
4. O callback Slint `WebviewLogic.loadUrl` solicita ao Servo a navegação para esse endereço.
5. O Servo carrega e renderiza o site completo.
6. O delegado recebe notificações de novos frames, pinta o WebView e atualiza a imagem apresentada na interface Slint.

Não existe chamada ao SearXNG, à API oficial DuckDuckGo, ao endpoint interno `i.js`, nem extração dos resultados HTML/JSON. A página exibida é o próprio site remoto.

## Eventos e interação

A área de navegação encaminha eventos de teclado, mouse, toque e rolagem para o WebView do Servo. As ações voltar, avançar e recarregar são encaminhadas pelos métodos do WebView. O Servo sinaliza novas atividades por um `EventLoopWaker`; o adaptador usa um canal smol para acordar a tarefa local do Slint e executar `spin_event_loop`.

## Renderização

O exemplo oficial integra Servo e Slint usando WGPU 3.0. O caminho GPU utiliza Surfman e exportação/importação de memória Vulkan no Linux para transportar o frame renderizado. Quando a criação do contexto GPU do Servo falha, o adaptador fornece um caminho alternativo de renderização por software e converte o framebuffer em uma imagem Slint.

## Limitações

- A integração Servo + Slint é experimental; não deve ser tratada como navegador pronto para produção.
- Sites complexos podem depender de APIs web ainda não implementadas pelo Servo.
- O protótipo mantém uma única WebView e não possui abas ou histórico persistente.
- Downloads, favoritos, controles de cookies, permissões, configurações de privacidade e ferramentas de desenvolvedor não fazem parte desta etapa.
- A pesquisa carrega o site em vez de normalizar resultados em objetos de imagem. A galeria anterior deixa de existir.

## Construção e plataforma

O protótipo é direcionado inicialmente ao Linux 64-bit. O Servo amplia sensivelmente o grafo de dependências Rust e exige ferramentas e bibliotecas nativas de compilação/renderização. Consulte a documentação atual do Servo e do Slint para preparar o ambiente da distribuição Linux.

## Referências de implementação

- [Slint — exemplo Servo](https://github.com/slint-ui/slint/tree/master/examples/servo)
- [Servo — documentação de embedding](https://book.servo.org/embedding/overview.html)
- [Servo — API WebView](https://doc.servo.org/servo/struct.WebView.html)
