# RUN FROM YE

Proyecto 1 de Graficas por Computadora.

`RUN FROM YE` es un juego en primera persona hecho en Rust con un motor de
raycasting. El mundo real del juego es un mapa 2D, pero se renderiza como una
escena 3D al estilo de motores clasicos como Wolfenstein 3D. El jugador debe
explorar un complejo abandonado, encontrar una tarjeta, activar un terminal,
abrir una puerta y llegar a la salida mientras una pared anomala patrulla el
mapa y lo persigue.

El proyecto no usa un motor 3D externo. La escena se dibuja manualmente en un
buffer de pixeles y luego se muestra en pantalla con `minifb`.

## Objetivo del juego

En cada nivel el jugador debe completar esta secuencia:

1. Encontrar la tarjeta de acceso.
2. Activar el switch o terminal.
3. Abrir la puerta bloqueada.
4. Llegar a la salida.

Durante la exploracion aparece un perseguidor, representado como una pared
anomala. Si el jugador se acerca demasiado, el perseguidor despierta y empieza
a seguirlo por el laberinto. Si lo alcanza, el jugador pierde una vida y vuelve
al punto inicial. Al quedarse sin vidas, aparece la pantalla de derrota.

## Como ejecutar

Requisitos:

- Rust instalado.
- Sistema compatible con ventanas de `minifb`.
- Dispositivo de audio opcional para escuchar musica y efectos.

Comandos:

```bash
cargo run
cargo check
cargo test
cargo fmt
```

El comando principal para probar el proyecto es:

```bash
cargo run
```

## Controles

| Accion | Tecla |
| --- | --- |
| Avanzar | `W` o flecha arriba |
| Retroceder | `S` o flecha abajo |
| Moverse a la izquierda | `A`, flecha izquierda o `Z` |
| Moverse a la derecha | `D`, flecha derecha o `C` |
| Girar camara | Mouse |
| Usar habilidad sonora | `Q` |
| Aceptar en menus | `Enter` o `Espacio` |
| Volver / salir | `Esc` |

## Caracteristicas implementadas

- Motor de raycasting para generar una vista 3D desde un mapa 2D.
- Renderizado manual sobre un buffer de pixeles.
- Texturas procedurales para paredes, metal, ruinas y compuerta.
- Sprites billboard para el perseguidor.
- Buffer de profundidad para ocultar sprites detras de paredes.
- Movimiento del jugador con colisiones circulares.
- Deslizamiento contra paredes al separar el movimiento en eje X y eje Y.
- Tres niveles definidos como mapas ASCII.
- Sistema de llave, switch, puerta y salida.
- Enemigo perseguidor con patrulla, deteccion, persecucion y ataque.
- Pathfinding por celdas usando busqueda en anchura.
- Habilidad sonora con cooldown que repele temporalmente al perseguidor.
- HUD con vidas, mensajes, FPS, estado de objetivo y cooldown.
- Minimap para visualizar jugador, mapa, objetivos y perseguidor.
- Audio con musica del perseguidor y efecto de habilidad.
- Menus: inicio, seleccion de nivel, instrucciones, descripcion, victoria y derrota.

## Enfoque tecnico

### Raycasting

El renderer lanza un rayo por cada columna de la pantalla. Cada rayo avanza por
la grilla del mapa hasta encontrar una pared o un obstaculo. Con la distancia al
impacto se calcula la altura de la columna que debe dibujarse, logrando la
ilusion de profundidad.

Este sistema esta implementado en `src/renderer.rs`.

Puntos importantes:

- El mapa es 2D, pero se proyecta como 3D.
- Cada columna de pixeles corresponde a un rayo.
- Se usa un buffer de profundidad para ordenar paredes y sprites.
- Las paredes usan texturas procedurales generadas por codigo.

### Mapa y colisiones

Los niveles se escriben como texto ASCII en `src/map.rs`. Cada caracter se
convierte en un tile:

| Simbolo | Significado |
| --- | --- |
| Espacio | Piso caminable |
| `+`, `-`, `|`, `#` | Pared |
| `m` | Metal |
| `r` | Ruinas |
| `k` | Tarjeta |
| `s` | Switch |
| `d` | Puerta |
| `g` | Salida |
| `p` | Spawn del jugador |
| `e` / `c` | Spawn del perseguidor |

La funcion `can_stand_at` revisa si una entidad puede ocupar una posicion. Para
evitar atravesar paredes en diagonal, se valida el radio de la entidad contra
varios puntos alrededor de su posicion.

### Jugador

El jugador vive en `src/player.rs`.

Su estado incluye:

- Posicion `x`, `y`.
- Angulo de camara.
- Vidas.
- Cooldown de la habilidad sonora.
- Punto de respawn.

El movimiento depende de `dt`, el tiempo entre frames, para que la velocidad sea
estable aunque cambien los FPS. El movimiento se divide en pasos pequenos y se
comprueba contra el mapa para mejorar las colisiones.

### Perseguidor

El perseguidor vive en `src/chaser.rs`.

Tiene varios estados:

- Dormido o patrullando.
- Activo cuando detecta al jugador.
- Repelido cuando el jugador usa la habilidad sonora.

Cuando esta activo, calcula una ruta hacia el jugador usando busqueda en anchura
por la grilla del mapa. Tambien recalcula la ruta periodicamente para adaptarse
al movimiento del jugador.

Eventos principales:

- `Spotted`: el perseguidor detecto al jugador.
- `Lost`: el jugador logro alejarse.
- `HitPlayer`: el perseguidor alcanzo al jugador.

### Audio

El audio esta implementado en `src/audio.rs` con `rodio`.

Incluye:

- Musica o sonido asociado al perseguidor.
- Audio espacial segun la posicion relativa entre jugador y perseguidor.
- Efecto de habilidad sonora.
- Pausa del perseguidor mientras se reproduce la habilidad.

Si no hay dispositivo de audio disponible, el juego continua funcionando sin
sonido.

## Estructura del proyecto

```text
src/
  main.rs       Punto de entrada del programa.
  config.rs     Constantes globales y valores de balance.
  game.rs       Loop principal, pantallas y flujo del juego.
  input.rs      Traduccion de teclado/mouse a acciones.
  player.rs     Movimiento, vida, respawn y habilidad sonora.
  map.rs        Niveles, tiles, colisiones e interacciones.
  chaser.rs     Logica del perseguidor y pathfinding.
  audio.rs      Musica, efectos y audio espacial.
  renderer.rs   Raycasting, sprites, minimap, HUD y menus.
  texture.rs    Texturas procedurales.

assets/
  chaser.png
  disgust_chase.png
  I Wonder.mp3
  The Fate of Ophelia.mp3
```

## Flujo principal del programa

1. `main.rs` crea una instancia de `Game`.
2. `Game::run` mantiene activo el loop principal.
3. Cada frame calcula `dt`.
4. Segun la pantalla actual, `Game` actualiza menus, juego, victoria o derrota.
5. Durante la partida se actualizan input, jugador, perseguidor, mapa y audio.
6. `Renderer` dibuja el frame completo en un buffer.
7. `minifb` muestra el buffer en la ventana.

La clase central del proyecto es `Game`, ubicada en `src/game.rs`, porque conecta
todos los demas sistemas.

## Dependencias

| Dependencia | Uso |
| --- | --- |
| `minifb` | Crear ventana, leer input y mostrar el buffer de pixeles |
| `rodio` | Reproducir audio y audio espacial |
| `image` | Cargar imagenes PNG/JPEG para sprites |
| `winapi` | Ajustes de ventana en Windows |

## Que revisar al calificar

Los puntos mas representativos del proyecto son:

- `src/renderer.rs`: implementacion del raycaster, HUD, minimap y sprites.
- `src/map.rs`: definicion de niveles ASCII, tiles, colisiones e interacciones.
- `src/chaser.rs`: IA del perseguidor, patrulla, deteccion y pathfinding.
- `src/player.rs`: movimiento con colisiones, vidas y habilidad sonora.
- `src/audio.rs`: audio espacial y sonidos del gameplay.

Este proyecto demuestra como construir una experiencia 3D interactiva usando
un mapa 2D, matematicas de raycasting, manejo manual de pixeles, colisiones,
estado de juego, pathfinding y audio.
