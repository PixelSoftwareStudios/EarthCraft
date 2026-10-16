# Navegação por controles

[English](CONTROLLERS.md) | [Português (Brasil)](CONTROLLERS.pt-BR.md)

Configurações → Controles permite usar gamepads reconhecidos pelo
[SDL3](https://wiki.libsdl.org/SDL3/CategoryGamepad), com conexão sem reiniciar.
Selecione o dispositivo, ajuste o perfil Explorar, acompanhe entradas ao vivo e
restaure os padrões. Mouse e teclado continuam disponíveis.

O analógico esquerdo move o globo; o eixo X direito gira e o eixo Y inclina a câmera; gatilhos
esquerdo/direito afastam/aproximam. Sul adiciona um marcador no centro, Leste
cancela movimento/arraste, Norte redefine orientação e inclinação, Oeste alterna medições,
Select abre Lugares e Start abre Configurações. Os nomes seguem posições
normalizadas, sem pressupor marca de console. A inclinação aplica-se apenas à Terra.

Cada ação de navegação permite escolher um dos seis eixos ou desativá-la, com
zona morta, inversão, sensibilidade e curva independentes. Eixos compartilhados
são permitidos explicitamente. Botões reconhecidos podem receber qualquer ação.
Pressionar para vincular aguarda dez segundos, permite cancelar, exige passagem
pelo neutro antes de capturar um eixo e confirma a substituição de botão ocupado.

O movimento usa tempo limitado e só é aplicado uma vez por frame. A Terra usa o
modelo selecionado e outros corpos usam a órbita existente. Ações reutilizam o
mesmo dispatcher dos menus. Desconectar libera entradas. Edição de texto,
diálogos, popups, janela sem foco e reinicialização suspendem movimento; solte
eixos/botões antes de retomar. A consulta limitada permite detectar conexão com
o globo parado.

Perfis locais em settings.json usam GUID SDL do modelo, fabricante/produto e nome.
Modelos diferentes mantêm perfis independentes; modelos idênticos sem identidade
única compartilham perfil. Não é identificação por número de série. Perfis
inválidos usam padrões independentes e números são limitados.

A compilação nativa exige CMake e compilador C/C++ para vincular estaticamente apenas
os subsistemas de gamepad/HID do SDL3. O driver Switch trata inicialização e calibração
USB. O monitor mostra todos os eixos e botões em linhas fixas. Linux/FreeBSD exigem
acesso aos dispositivos. Snap solicita joystick e Flatpak, dispositivos de entrada.
Permissões podem exigir configuração. Erros aparecem nas Configurações. Compilar
no macOS não comprova pacotes ou funcionamento físico.

Studio/voo, HOTAS/HID bruto, SpaceMouse, vibração e importação/exportação de perfis
ficam para trabalho posterior. Os testes exercitam zona morta, inversão/curvas,
bordas/liberação de botões, conflitos, passos limitados e persistência por modelo.

A inclinação varia de 0° (nadir) a 85° em direção ao horizonte, mantendo posição e
altitude da câmera. Em altitude elevada, o centro pode apontar para o espaço.
Arraste com o botão central ou use o controle no painel Globo. R redefine norte e
inclinação; N redefine apenas orientação. Voar até um ponto retorna suavemente ao
nadir. A vista inicial salva mantém a inclinação. Perfis antigos de cinco ações
mantêm vínculos e recebem inclinação no eixo Y direito. Perfis anteriores do mesmo
fabricante/produto são migrados. Testes físicos USB/Bluetooth ainda são necessários.
