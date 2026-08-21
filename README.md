# RUN FROM YE

Proyecto de graficas hecho en Rust. El juego es un raycaster: usa un mapa 2D
para dibujar una escena con apariencia 3D, al estilo de motores clasicos como
Wolfenstein 3D. El jugador debe explorar un complejo, recoger una tarjeta,
activar un terminal, abrir la puerta y llegar a la salida mientras una pared
anomala lo patrulla y lo persigue.

Este README esta pensado como guia de estudio para explicar el proyecto,
entender donde esta cada funcionalidad y poder responder preguntas sobre la
implementacion.

## Como ejecutar

Requisitos:

- Rust instalado.
- Sistema con soporte para abrir ventana usando `minifb`.
- Dispositivo de audio si se quiere escuchar la musica y efectos.

Comandos utiles:

```bash
cargo run
cargo check
cargo fmt
```

`cargo run` compila y ejecuta el juego. `cargo check` verifica que el proyecto
compile sin generar el ejecutable final. `cargo fmt` formatea el codigo.

## Dependencias principales

Estan definidas en `Cargo.toml`.

- `minifb`: crea la ventana, lee teclado/mouse y muestra el buffer de pixeles.
- `rodio`: reproduce audio, incluyendo audio espacial para el perseguidor.
- `image`: carga imagenes externas para el sprite del chaser.
- `winapi`: se usa en Windows para consultar resolucion de pantalla y recentrar
  el mouse en modo fullscreen/borderless.

## Estructura del proyecto

```text
src/
  main.rs       Punto de entrada.
  config.rs     Constantes globales del juego.
  game.rs       Loop principal, pantallas y orquestacion.
  input.rs      Traduccion de teclado/mouse a acciones.
  player.rs     Movimiento, vida, respawn y habilidad sonora.
  map.rs        Niveles, tiles, colisiones e interacciones.
  chaser.rs     Logica de la pared perseguidora.
  audio.rs      Musica del chaser y sonido de habilidad.
  renderer.rs   Raycasting, sprites, minimapa, HUD y menus.
  texture.rs    Texturas procedurales de paredes.

assets/
  chaser.png
  disgust_chase.png
  I Wonder.mp3
  The Fate of Ophelia.mp3
```

## Idea general del juego

El jugador empieza en un nivel laberintico. El objetivo es:

1. Encontrar la tarjeta de acceso.
2. Activar el terminal/switch.
3. Abrir la puerta.
4. Llegar a la salida.

Mientras tanto, existe un chaser o perseguidor. Es una pared/anomalia que puede
patrullar, detectar al jugador, perseguirlo usando pathfinding y golpearlo. Si
lo golpea, el jugador pierde vida. Cuando la vida llega a `0`, aparece la
pantalla:

```text
YE TE AH ATRAPADO
```

Desde ahi se puede:

- Seguir con el mismo nivel.
- Volver al menu principal.

## Controles

Implementados principalmente en `src/input.rs` y usados por `src/player.rs`.

- `W` o flecha arriba: avanzar.
- `S` o flecha abajo: retroceder.
- `A`, flecha izquierda o `Z`: moverse lateralmente a la izquierda.
- `D`, flecha derecha o `C`: moverse lateralmente a la derecha.
- Mouse: girar la camara.
- `Q`: usar habilidad sonora.
- `Enter` o `Espacio`: aceptar opciones en menus.
- `Esc`: volver al menu o salir segun la pantalla.

## Flujo principal del programa

El flujo empieza en `src/main.rs`.

1. `main.rs` declara los modulos.
2. Crea el juego con `Game::new()`.
3. Ejecuta el loop con `game.run()`.
4. Cada frame calcula `dt`, que es el tiempo entre frames.
5. Segun la pantalla actual, `Game` actualiza menus, gameplay, exito o derrota.
6. El renderer dibuja el frame en un buffer.
7. `minifb` muestra ese buffer en la ventana.

La estructura central es `Game`, definida en `src/game.rs`.

## Pantallas del juego

Las pantallas estan representadas por el enum `GameScreen` en `src/game.rs`.

- `MainMenu`: menu principal.
- `LevelSelect`: seleccion de niveles.
- `Instructions`: instrucciones.
- `About`: contexto narrativo.
- `Playing`: partida activa.
- `LevelSuccess`: pantalla al completar nivel.
- `GameOver`: pantalla cuando la vida llega a `0`.

El metodo `update_screen` decide que funcion ejecutar segun la pantalla actual.

## `src/main.rs`

Es el archivo de entrada.

Responsabilidades:

- Declarar todos los modulos del proyecto.
- Crear `Game`.
- Llamar `run`.

Codigo clave:

- `let mut game = Game::new()?;`
- `game.run()`

Si el profesor pregunta por donde arranca todo, la respuesta es: el programa
arranca en `main.rs`, pero la logica real vive en `Game`.

## `src/config.rs`

Centraliza constantes para no tener numeros magicos repartidos por el codigo.

Constantes importantes:

- `SCREEN_WIDTH` y `SCREEN_HEIGHT`: tamano del buffer interno.
- `BORDERLESS_FULLSCREEN`: decide si la ventana usa pantalla completa sin borde.
- `TARGET_FPS`: FPS objetivo.
- `MOVE_SPEED`: velocidad del jugador.
- `MOUSE_SENSITIVITY`: sensibilidad del giro con mouse.
- `PLAYER_RADIUS`: radio usado para colisiones del jugador.
- `FOV_FACTOR`: campo de vision del raycaster.
- `SOUND_ABILITY_COOLDOWN`: tiempo antes de poder volver a usar `Q`.
- `SOUND_ABILITY_RANGE`: distancia maxima a la que el sonido afecta al chaser.
- `CHASER_SPEED`: velocidad del chaser persiguiendo.
- `CHASER_WAKE_DISTANCE`: distancia a la que detecta al jugador.
- `CHASER_LOSE_DISTANCE`: distancia a la que deja de perseguir.
- `CHASER_HIT_DISTANCE`: distancia a la que golpea al jugador.
- `CHASER_REPATH_INTERVAL`: cada cuanto recalcula ruta.
- `CHASER_DISGUST_DURATION`: duracion del estado repelido.
- `CHASER_DISGUST_SPEED`: velocidad cuando retrocede por la habilidad.
- `MESSAGE_DURATION`: duracion de mensajes temporales del HUD.
- `MINIMAP_CELL_SIZE` y `MINIMAP_PADDING`: configuracion del minimapa.

Ventaja de este archivo: permite balancear el juego sin tocar la logica.

## `src/game.rs`

Es el orquestador principal. No dibuja pixeles directamente ni calcula el
pathfinding, pero conecta todos los sistemas.

### Estructura `Game`

Campos principales:

- `window`: ventana de `minifb`.
- `renderer`: sistema que dibuja todo.
- `map`: nivel actual.
- `player`: jugador.
- `chaser`: perseguidor.
- `audio`: sistema de audio.
- `previous_frame`: tiempo del frame anterior.
- `fps_timer`, `frame_counter`, `displayed_fps`: medicion de FPS.
- `message`, `message_timer`: texto temporal del HUD.
- `mouse_centered`: evita saltos del mouse al iniciar/recentrar.
- `screen`: pantalla activa.
- `menu_selection`: opcion del menu principal.
- `level_selection`: opcion del selector de niveles.
- `game_over_selection`: opcion de la pantalla de derrota.
- `current_level`: nivel actual.

### `Game::new`

Inicializa:

- Ventana.
- Renderer.
- Primer mapa.
- Spawn del jugador.
- Spawn del chaser.
- Audio.
- Estado inicial de menus.

Tambien configura fullscreen/borderless si esta activado.

### `Game::run`

Es el loop principal:

1. Mientras la ventana este abierta.
2. Calcula `dt`.
3. Llama `update_screen`.
4. Manda el buffer del renderer a la ventana con `update_with_buffer`.

`dt` es importante porque permite que el movimiento dependa del tiempo real y
no de la velocidad de la computadora.

### Menus y pantallas

Funciones relevantes:

- `update_main_menu`: mueve seleccion y permite ir a jugar/instrucciones/about/salir.
- `update_level_select`: elige nivel y llama `start_new_game`.
- `update_instructions`: muestra controles.
- `update_about`: muestra contexto.
- `update_level_success`: permite continuar al siguiente nivel o volver al menu.
- `update_game_over`: permite reintentar el mismo nivel o volver al menu.

### Partida activa: `update_playing`

Este metodo corre cuando `screen == GameScreen::Playing`.

Orden del frame:

1. Revisa `Esc` para volver al menu.
2. Lee movimiento del mouse.
3. Construye `InputState`.
4. Actualiza al jugador.
5. Si se uso `Q`, llama `activate_disgust_sound`.
6. Actualiza el chaser con `update_chaser`.
7. Si el chaser dejo la vida en `0`, cambia a `GameOver`.
8. Actualiza interacciones del mapa.
9. Actualiza FPS.
10. Actualiza audio del chaser.
11. Renderiza escena, minimapa y HUD.

### Carga de niveles

Funciones:

- `start_new_game(level_index)`
- `load_level(level_index)`

`load_level` reinicia:

- Mapa.
- Jugador.
- Chaser.
- Mensaje de objetivo.
- Timer de frame.
- Estado del mouse.
- Pantalla a `Playing`.

Por eso al elegir "SEGUIR MISMO NIVEL" en Game Over se llama `load_level` con
el mismo `current_level`.

### Mouse relativo

Funciones:

- `read_mouse_delta_x`
- `center_mouse`
- `mouse_delta_x_from_center`
- `window_center`

Como `minifb` no da movimiento relativo estilo FPS directamente, el juego
recoloca el cursor al centro de la ventana cada frame y calcula cuanto se movio
desde ese centro. Ese delta horizontal gira la camara.

### FPS

`update_fps` cuenta frames y cada segundo actualiza el titulo de la ventana con
el FPS actual.

### Mensajes de objetivo

`current_goal_message` decide el mensaje persistente:

- Sin llave: `FIND ACCESS CARD`.
- Con llave pero sin switch: `ACTIVATE TERMINAL`.
- Con puerta abierta: `REACH EXIT`.
- Nivel completado: `SITE SECURED`.

## `src/input.rs`

Traduce entradas concretas de `minifb` a acciones de juego.

La estructura `InputState` contiene:

- `move_forward`
- `move_backward`
- `mouse_delta_x`
- `strafe_left`
- `strafe_right`
- `sound_ability`

La ventaja es separar la entrada fisica de la logica del jugador. `player.rs`
no necesita saber que libreria lee el teclado; solo recibe acciones.

Funcion principal:

- `InputState::from_window(window, mouse_delta_x)`

## `src/player.rs`

Contiene estado y movimiento del jugador.

### Campos de `Player`

- `x`, `y`: posicion en coordenadas del mapa.
- `angle`: direccion de mirada en radianes.
- `lives`: vidas actuales.
- `sound_ability_cooldown`: cooldown de la habilidad sonora.
- `spawn_x`, `spawn_y`, `spawn_angle`: punto de respawn.

### Creacion

`Player::new(x, y, angle)` crea al jugador con:

- Posicion inicial.
- Angulo inicial.
- `3` vidas.
- Cooldown en `0`.
- Spawn igual a la posicion inicial.

### Movimiento

`Player::update(input, map, dt)` hace:

1. Reduce cooldown de habilidad.
2. Aplica rotacion con `mouse_delta_x * MOUSE_SENSITIVITY`.
3. Calcula vector frontal usando `cos(angle)` y `sin(angle)`.
4. Calcula vector lateral perpendicular.
5. Suma desplazamiento segun teclas.
6. Llama `move_with_collision`.
7. Si se presiono `Q` y no hay cooldown, devuelve `true`.

Ese `true` le avisa a `Game` que debe activar el sonido de disgusto.

### Colisiones del jugador

`move_with_collision` divide el movimiento grande en pasos pequenos usando
`COLLISION_STEP`. Esto evita atravesar paredes si el jugador se mueve rapido.

`try_move` intenta mover en X y luego en Y. Separar los ejes permite deslizarse
por paredes en lugar de detenerse completamente.

La validacion real de si se puede estar en una posicion se delega a:

- `Map::can_stand_at`

### Vida y respawn

`take_hit_and_respawn` resta una vida con `saturating_sub(1)`.

- Si quedan vidas, hace respawn.
- Si la vida queda en `0`, no respawnea; `Game` cambia a pantalla `GameOver`.

`respawn` restaura:

- `x`
- `y`
- `angle`

## `src/map.rs`

Define niveles, tiles, colisiones e interacciones.

### Formato maze

Los mapas ya no se escriben como numeros. Ahora se escriben como un maze ASCII,
parecido a este formato:

```text
+-+-+
|p k|
+ + +
|e dg
+---+
```

Leyenda:

- `espacio`: piso caminable.
- `+`, `-`, `|` o `#`: pared normal.
- `p`: spawn del jugador.
- `e` o `c`: spawn del chaser/enemigo.
- `d`: puerta/gate.
- `m`: pared/obstaculo metalico.
- `r`: ruinas/obstaculo.
- `k`: tarjeta/llave.
- `s`: switch/terminal.
- `g`: salida/meta.

El parser tambien acepta mayusculas para los simbolos especiales. Si aparece un
caracter desconocido, se trata como pared por seguridad.

Internamente el juego todavia usa tiles numericos, porque el renderer, las
colisiones y el pathfinding trabajan mejor con una grilla compacta de `u8`. El
maze es la forma humana y legible de escribir el mapa; `from_maze` lo convierte
al formato interno.

### Niveles

Funciones:

- `Map::level_count()`
- `Map::level(index)`
- `Map::level_one()`
- `Map::level_two()`
- `Map::level_three()`

Cada nivel se define como arreglo de strings tipo maze. Ejemplo conceptual:

```text
+---+
|p g|
+---+
```

Luego `from_maze` convierte cada caracter a su tile correspondiente y guarda
todo en `tiles`.

El maze tambien define:

- `player_spawn`: sale del caracter `p` y recibe un angulo inicial por parametro.
- `chaser_spawn`: sale del caracter `e` o `c`.

### Estado del mapa

`Map` guarda:

- `width`
- `height`
- `tiles`
- `has_key`
- `switch_pressed`
- `completed`
- `player_spawn`
- `chaser_spawn`

### Lectura de tiles

`tile_at(x, y)` devuelve el tile de una celda. Si se consulta fuera del mapa,
devuelve pared. Esto evita errores y tambien impide salir del nivel.

`displayed_tile_at(x, y)` devuelve el tile que se debe usar para dibujar y
colisionar segun el estado actual. Ejemplo: si la puerta ya esta abierta,
`TILE_GATE` se convierte en `TILE_FLOOR`.

### Colisiones

`can_stand_at(x, y, radius)` revisa cuatro puntos alrededor de la entidad:

- esquina superior izquierda del radio.
- esquina superior derecha.
- esquina inferior izquierda.
- esquina inferior derecha.

Si todos esos puntos estan en tiles no bloqueantes, la posicion es valida.

`is_walkable_cell(x, y)` se usa para entidades por celdas, especialmente el
chaser y su pathfinding.

`blocks_player(tile)` define que bloquea:

- Piso, tarjeta, switch y salida no bloquean.
- Puerta bloquea solo si esta cerrada.
- Pared, metal y ruinas bloquean.
- Tiles desconocidos bloquean.

### Interacciones

`update_player_interactions(player)` revisa el tile donde esta parado el
jugador.

Casos:

- Si pisa `TILE_KEY`, recoge la tarjeta, cambia el tile a piso y devuelve
  `ACCESS CARD FOUND`.
- Si pisa `TILE_SWITCH` con tarjeta, activa el switch y devuelve
  `DOOR UNLOCKED`.
- Si pisa `TILE_SWITCH` sin tarjeta, devuelve `NEED ACCESS CARD`.
- Si pisa `TILE_EXIT` con puerta abierta, marca el nivel como completado.
- Si pisa `TILE_EXIT` sin puerta abierta, devuelve `EXIT SEALED`.

## `src/chaser.rs`

Contiene la logica del perseguidor, que es una de las partes mas importantes
del juego.

### Estados principales

El chaser puede estar:

- Dormido/patrullando: no persigue al jugador, se mueve por pasillos.
- Activo: detecto al jugador y lo persigue.
- Disgusted/repelido: fue afectado por la habilidad sonora y retrocede.

Campos principales de `Chaser`:

- `x`, `y`: posicion actual.
- `spawn_x`, `spawn_y`: posicion inicial.
- `active`: si esta persiguiendo.
- `disgust_timer`: tiempo restante de repelido.
- `path`: ruta calculada hacia el jugador.
- `repath_timer`: tiempo hasta recalcular ruta.
- `patrol_target`: celda objetivo mientras patrulla.
- `patrol_direction`: direccion preferida de patrulla.
- `visit_counts`: cuantas veces visito cada celda.
- `rng_state`: estado pseudoaleatorio para variar la patrulla.

### Eventos del chaser

El enum `ChaserEvent` puede devolver:

- `Spotted`: detecto al jugador.
- `Lost`: perdio al jugador.
- `HitPlayer`: alcanzo y golpeo al jugador.

Estos eventos los recibe `Game::update_chaser`, que actualiza mensajes, vida,
respawn, audio y Game Over.

### Actualizacion general

Funcion principal:

- `Chaser::update(map, player, dt)`

Orden logico:

1. Si el mapa ya esta completado, se desactiva.
2. Calcula distancia al jugador.
3. Si esta repelido, baja `disgust_timer` y llama `repel_from_player`.
4. Si no esta activo, patrulla.
5. Si el jugador entra en `CHASER_WAKE_DISTANCE`, se activa y devuelve `Spotted`.
6. Si esta activo y el jugador se aleja mas de `CHASER_LOSE_DISTANCE`, devuelve `Lost`.
7. Si sigue activo, recalcula ruta cada `CHASER_REPATH_INTERVAL`.
8. Sigue la ruta.
9. Si queda a `CHASER_HIT_DISTANCE`, devuelve `HitPlayer`.

### Deteccion

La deteccion se basa en distancia:

- Si esta dormido y el jugador entra al radio `CHASER_WAKE_DISTANCE`, despierta.
- Si esta activo y el jugador se aleja hasta `CHASER_LOSE_DISTANCE`, se calma.

Esto produce una persecucion simple pero efectiva.

### Pathfinding

La ruta se calcula en `find_path`.

Usa BFS, busqueda en anchura:

1. Toma una celda inicial: posicion del chaser.
2. Toma una celda objetivo: posicion del jugador.
3. Explora vecinos arriba, abajo, izquierda y derecha.
4. Solo entra a celdas caminables segun `Map::is_walkable_cell`.
5. Guarda de donde vino cada celda en `came_from`.
6. Cuando llega al objetivo, reconstruye el camino.

BFS es adecuado aqui porque el mapa es una grilla sin pesos: cada paso entre
celdas cuesta lo mismo.

### Movimiento por ruta

`follow_path` toma la primera celda de `path`, calcula su centro y mueve el
chaser hacia ese punto.

Si llega cerca del centro, elimina esa celda de la ruta y continua con la
siguiente.

Si el movimiento choca con el mapa, limpia la ruta para forzar un recalculo.

### Patrulla

Cuando no esta persiguiendo, usa:

- `patrol`
- `next_patrol_cell`
- `visit_counts`

La patrulla intenta elegir entre avanzar, girar a la derecha o girar a la
izquierda. Prefiere celdas menos visitadas para no quedarse dando vueltas en el
mismo lugar. Si no puede avanzar, intenta retroceder o tomar una direccion
fallback.

### Habilidad sonora contra el chaser

`Chaser::disgust(player)` revisa si la distancia al jugador es menor o igual a
`SOUND_ABILITY_RANGE`.

Si el sonido alcanza:

- Activa al chaser.
- Pone `disgust_timer`.
- Limpia la ruta.
- Cancela objetivo de patrulla.
- Devuelve `true`.

Luego `repel_from_player` calcula la direccion opuesta al jugador y mueve al
chaser alejandolo. Si no puede moverse directo por una pared, prueba direcciones
laterales.

## `src/audio.rs`

Controla musica y efectos.

Usa `rodio`.

### Archivos usados

- `assets/I Wonder.mp3`: audio del chaser.
- `assets/The Fate of Ophelia.mp3`: sonido de habilidad/disgusto.

Constantes internas:

- `DISGUST_AUDIO_DURATION`: duracion del efecto de habilidad.
- `DISGUST_AUDIO_VOLUME`: volumen del efecto.
- `CHASER_MAX_VOLUME`: volumen maximo del chaser.
- `CHASER_AUDIBLE_DISTANCE`: distancia maxima audible.
- `EAR_DISTANCE`: separacion simulada entre oidos para audio espacial.

### `AudioSystem`

Campos:

- `_device_sink`: mantiene vivo el dispositivo/mixer.
- `chaser_player`: reproductor espacial del chaser.
- `ability_player`: reproductor normal para la habilidad.
- `ability_until`: instante hasta el que dura la habilidad.

### Inicializacion

`AudioSystem::new()` intenta abrir el dispositivo de audio.

Si no puede, imprime un mensaje y crea un audio desactivado para que el juego no
se cierre. Esto es importante: el audio falla de forma tolerante.

Si puede:

1. Crea `SpatialPlayer` para el chaser.
2. Crea `Player` para la habilidad.
3. Carga el loop del chaser.
4. Lo pausa hasta que sea audible.

### Audio espacial del chaser

`update_chaser_audio(player, chaser, playing)` calcula:

- Distancia del chaser al jugador.
- Posicion relativa respecto al angulo del jugador.
- Volumen segun cercania.
- Boost si el chaser esta activo.
- Velocidad de reproduccion un poco distinta si esta activo o no.

El audio se pausa si:

- No se esta jugando.
- La habilidad sonora esta sonando.
- El volumen seria practicamente cero.

### Sonido de habilidad

`play_disgust_sound()`:

1. Actualiza el timer de habilidad.
2. Si ya esta sonando, no la reinicia.
3. Pausa el audio del chaser.
4. Carga el MP3 de habilidad.
5. Lo reproduce con duracion y volumen limitados.
6. Marca `ability_until`.

### Busqueda de assets

`asset_path` y `asset_roots` buscan archivos en:

- Carpeta `assets` del proyecto.
- Carpeta actual.
- Carpeta del ejecutable.

Esto ayuda a que los assets funcionen tanto en desarrollo como al ejecutar el
binario desde otra ubicacion.

## `src/renderer.rs`

Es el modulo mas grande. Dibuja todo en un buffer de pixeles `Vec<u32>`.

No usa un motor 3D real. La escena se calcula manualmente con raycasting.

### Estructura `Renderer`

Campos:

- `width`, `height`: tamano del buffer.
- `buffer`: pixeles RGB que se muestran en la ventana.
- `background`: fondo precalculado de cielo/piso.
- `depth_buffer`: distancia a pared por columna.
- `chaser_texture`: imagen externa opcional del chaser.
- `disgust_chaser_texture`: imagen externa opcional del chaser repelido.

### Render principal

`render(map, player, chaser, fps, message)` dibuja en este orden:

1. Copia el fondo.
2. Dibuja paredes con `draw_walls`.
3. Dibuja sprites con `draw_sprites`.
4. Dibuja minimapa con `draw_minimap`.
5. Dibuja HUD con `draw_hud`.

El orden importa porque el HUD debe quedar encima de la escena.

### Pantallas de menu

Funciones:

- `render_main_menu`
- `render_level_select_screen`
- `render_instructions_screen`
- `render_about_screen`
- `render_level_success_screen`
- `render_game_over_screen`

Todas usan primitivas internas como:

- `draw_menu_background`
- `draw_menu_button`
- `draw_info_panel`
- `draw_centered_text`

### Raycasting de paredes

Implementado en:

- `draw_walls`
- `cast_ray`

Idea:

Para cada columna horizontal de pantalla se lanza un rayo desde el jugador.
Ese rayo avanza por el mapa 2D hasta encontrar una pared. Con la distancia a la
pared se calcula que tan alta debe verse esa pared en pantalla.

Pasos en `draw_walls`:

1. Calcula la direccion del jugador.
2. Calcula el plano de camara usando `FOV_FACTOR`.
3. Para cada columna `screen_x`, calcula un `camera_x` entre `-1` y `1`.
4. Construye direccion del rayo.
5. Llama `cast_ray`.
6. Guarda distancia en `depth_buffer`.
7. Calcula altura visible de pared.
8. Calcula coordenada vertical de textura.
9. Usa `wall_texel` de `texture.rs`.
10. Aplica sombreado con `shade_color`.
11. Pinta la columna.

### DDA en `cast_ray`

`cast_ray` usa DDA, Digital Differential Analyzer.

La idea es no avanzar pixel por pixel, sino celda por celda en el mapa:

1. Calcula en que celda empieza el jugador.
2. Calcula cuanto cuesta cruzar una celda en X y en Y.
3. Decide si el siguiente cruce es vertical u horizontal.
4. Avanza a la siguiente celda.
5. Revisa si esa celda contiene pared/gate/metal/ruinas.
6. Al chocar, calcula distancia perpendicular para evitar efecto ojo de pez.
7. Calcula `texture_x` para saber que columna de textura usar.

`RayHit` devuelve:

- `distance`: distancia perpendicular a la pared.
- `wall_id`: tile golpeado.
- `side`: lado golpeado, usado para sombreado.
- `texture_x`: coordenada horizontal de textura.

### Sprites

Implementado en:

- `draw_sprites`
- `sample_sprite_color`
- `sprite_color`
- `SpriteTexture::sample`

Sprites dibujados:

- Tarjeta.
- Switch.
- Salida.
- Chaser.

El chaser puede usar imagen externa:

- Normal: `chaser.png`.
- Repelido: `disgust_chase.png`.

Si no encuentra imagen, usa sprite procedural.

Como los sprites son planos dentro de un mundo 3D falso, se transforman a
espacio de camara y se escalan segun distancia. Luego se dibujan de lejos a
cerca para que se superpongan bien.

El `depth_buffer` evita que un sprite se dibuje encima de una pared si la pared
esta mas cerca que el sprite.

### Minimap

`draw_minimap` dibuja:

- Celdas del mapa.
- Jugador como circulo blanco.
- Direccion del jugador como linea.
- Chaser como rectangulo.
- Estado del chaser por color.
- Indicador circular del cooldown de `Q`.

Los colores de tiles salen de `minimap_color`.

### HUD

`draw_hud` muestra:

- FPS.
- HP/vidas.
- Si tiene tarjeta.
- Estado de la puerta.
- Estado del chaser.
- Mensaje actual.
- Controles basicos.

### Texto

El texto no usa una fuente externa. Se dibuja con una fuente bitmap de 3x5
pixeles.

Funciones:

- `draw_text`
- `draw_centered_text`
- `draw_char`
- `glyph`
- `text_pixel_width`

`glyph` contiene el patron binario de cada letra o numero.

### Primitivas de dibujo

Funciones de bajo nivel:

- `put_pixel`: pinta un pixel.
- `fill_rect`: pinta rectangulo.
- `fill_circle`: pinta circulo.
- `fill_clockwise_circle_slice`: pinta una porcion circular.
- `draw_line`: pinta linea con Bresenham.

Estas son la base para menus, HUD, minimapa y sprites procedurales.

### Texturas externas del chaser

Funciones:

- `load_chaser_texture`
- `chaser_texture_paths`
- `load_sprite_texture`
- `chaser_tint`
- `is_chaser_texture_transparent`

El renderer intenta cargar imagenes desde `assets`. Si no puede, sigue con una
version procedural. Los pixeles con alpha bajo se convierten en magenta
`0xff00ff`, que luego se trata como transparente.

## `src/texture.rs`

Genera texturas procedurales para paredes y obstaculos.

Funcion publica principal:

- `wall_texel(tile, x, y)`

Recibe:

- Tipo de tile.
- Coordenada X dentro de textura.
- Coordenada Y dentro de textura.

Devuelve un color RGB en formato `0xRRGGBB`.

Texturas:

- `stone_brick`: pared normal tipo concreto/ladrillo.
- `gate`: puerta con franjas de peligro y remaches.
- `metal_panel`: panel metalico con uniones y rayones.
- `cracked_ruins`: ruinas con grietas y crecimiento organico.
- `fallback`: patron generico si el tile no es reconocido.

`TEXTURE_SIZE` es `64`, asi que las texturas virtuales son de 64x64.

`noise` genera variaciones deterministas para que las paredes no sean planas.
No es aleatorio real en runtime; para la misma coordenada devuelve siempre el
mismo valor.

## Como se conectan los sistemas

Flujo de una persecucion:

1. `Game::update_playing` actualiza al jugador.
2. `Game::update_chaser` llama `Chaser::update`.
3. `Chaser::update` revisa distancia, patrulla, pathfinding o golpe.
4. Si devuelve `HitPlayer`, `Game` llama `player.take_hit_and_respawn`.
5. Si todavia hay vidas, reinicia el chaser.
6. Si la vida llega a `0`, cambia a `GameScreen::GameOver`.
7. `Renderer::render_game_over_screen` dibuja la pantalla de derrota.

Flujo de la habilidad sonora:

1. El jugador presiona `Q`.
2. `InputState` marca `sound_ability`.
3. `Player::update` revisa cooldown y devuelve `true`.
4. `Game::activate_disgust_sound` reproduce audio.
5. `Chaser::disgust` revisa si esta en rango.
6. Si esta en rango, el chaser entra a estado repelido.
7. Mientras `disgust_timer > 0`, `Chaser::update` llama `repel_from_player`.

Flujo para completar nivel:

1. El jugador pisa la tarjeta.
2. `Map::update_player_interactions` pone `has_key = true`.
3. El jugador pisa el switch.
4. Si tiene tarjeta, `switch_pressed = true`.
5. `gate_open()` empieza a devolver `true`.
6. `displayed_tile_at` trata la puerta como piso.
7. El jugador pisa la salida.
8. `completed = true`.
9. `Game` cambia a `LevelSuccess`.

## Preguntas probables del profesor

### Que es raycasting?

Es una tecnica donde se lanza un rayo por cada columna de pantalla desde la
posicion del jugador. El rayo avanza por el mapa 2D hasta encontrar una pared.
La distancia a esa pared determina la altura de la columna dibujada, creando la
ilusion de 3D.

En este proyecto esta en `renderer.rs`, especialmente en `draw_walls` y
`cast_ray`.

### Por que se usa DDA?

Porque el mundo esta organizado en una grilla. DDA permite avanzar de celda en
celda de forma eficiente hasta encontrar una pared, sin probar cada pixel o cada
punto continuo.

### Como se evita el efecto ojo de pez?

`cast_ray` calcula distancia perpendicular a la pared, no simplemente la
distancia directa del rayo. Eso mantiene las paredes rectas y evita distorsion
exagerada.

### Como funciona el chaser?

Tiene tres comportamientos:

- Patrulla cuando no detecta al jugador.
- Persigue cuando el jugador entra al radio de deteccion.
- Retrocede cuando la habilidad sonora lo afecta.

Para perseguir usa BFS sobre las celdas caminables del mapa.

### Por que BFS y no A*?

BFS es suficiente porque el mapa es una grilla sin pesos: moverse a cualquier
celda vecina cuesta lo mismo. A* podria ser mas eficiente en mapas enormes, pero
para estos niveles BFS es simple, correcto y facil de explicar.

### Como sabe el chaser donde moverse?

`find_path` devuelve una lista de celdas desde el chaser hasta el jugador.
`follow_path` toma la primera celda de esa lista y mueve al chaser hacia su
centro.

### Como funciona la habilidad de sonido?

El jugador presiona `Q`. Si el cooldown esta listo, `Player::update` devuelve
`true`. Luego `Game` reproduce el sonido y llama `Chaser::disgust`. Si el
chaser esta dentro del rango, entra en estado repelido durante unos segundos.

### Como funciona el audio espacial?

`AudioSystem::update_chaser_audio` calcula la posicion relativa del chaser
respecto al jugador usando el angulo de vista. Luego actualiza la posicion del
emisor en `rodio::SpatialPlayer`. Tambien baja el volumen segun distancia.

### Como se hacen las colisiones?

El jugador tiene un radio. `Map::can_stand_at` revisa varios puntos alrededor
de ese radio. Si alguno cae en una pared, la posicion no es valida.

Ademas, el movimiento se divide en pasos pequenos para no atravesar paredes por
ir muy rapido.

### Como se abre la puerta?

La puerta se abre logicamente cuando:

```rust
has_key && switch_pressed
```

Eso esta en `Map::gate_open`. Cuando devuelve `true`, `displayed_tile_at`
convierte la puerta en piso para dibujo y colision.

### Como se dibuja el texto?

No se usa una fuente externa. `renderer.rs` tiene una fuente bitmap manual en
`glyph`. Cada caracter es una matriz de 3x5 pixeles escalada.

### Que pasa si no hay audio o assets?

El juego no se cierra. Si no hay dispositivo de audio, `AudioSystem` queda
desactivado. Si no encuentra imagen del chaser, el renderer usa un sprite
procedural.

## Resumen corto para explicar en clase

El proyecto esta dividido por responsabilidades. `Game` maneja el flujo general
y las pantallas. `Player` controla movimiento, vida y habilidad. `Map` contiene
los niveles, tiles, colisiones e interacciones. `Chaser` maneja la IA del
perseguidor, incluyendo patrulla, deteccion, pathfinding BFS y golpe al
jugador. `AudioSystem` reproduce musica espacial y el efecto de habilidad.
`Renderer` dibuja todo manualmente en un buffer usando raycasting, sprites,
minimapa, HUD y menus. `texture.rs` genera las texturas procedurales de las
paredes.

La parte central de graficas es el raycasting: por cada columna de pantalla se
lanza un rayo sobre el mapa 2D, se detecta la pared mas cercana y se dibuja una
columna escalada segun distancia. La parte central de gameplay es el ciclo
jugador-mapa-chaser: el jugador explora, el mapa responde a interacciones y el
chaser patrulla o persigue usando la grilla del nivel.
