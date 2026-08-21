# Guía de estudio — RUN FROM YE (Proyecto 1 Gráficas)

## 1. Qué es el proyecto, en una frase

Un **raycaster estilo Wolfenstein 3D** escrito en Rust puro (sin motor gráfico), que dibuja un mundo 3D a partir de un mapa 2D lanzando un rayo por cada columna de pantalla. Es un juego de terror/escape: hay que encontrar una tarjeta, activar un switch, abrir una puerta y llegar a la salida mientras una "pared perseguidora" (el enemigo, llamado *Ye*) patrulla y puede activarse por cercanía.

**Stack:** `minifb` (ventana + framebuffer crudo, sin GPU), `image` (cargar PNG/JPG para sprites), `rodio` (audio espacial), `winapi` (detalles de ventana en Windows).

Todo el render es **software rendering**: vos escribís directo un `Vec<u32>` de píxeles (0xRRGGBB) y se lo pasás a `minifb`. No hay OpenGL/Vulkan/DirectX de por medio.

---

## 2. Arquitectura de módulos

```
main.rs      → arranca Game::new() y Game::run()
game.rs      → el "orquestador": loop principal, máquina de estados de pantallas,
               conecta input + player + chaser + map + renderer + audio
input.rs     → traduce teclado/mouse de minifb a un InputState neutral
player.rs    → posición, ángulo, colisión, vida, cooldown de habilidad
map.rs       → grilla de tiles, reglas de colisión, llave/switch/puerta/salida
chaser.rs    → IA del enemigo: patrulla, detección, pathfinding (BFS), disgusto
renderer.rs  → TODO el dibujo: raycasting de paredes, sprites, minimapa, HUD, menús
texture.rs   → texturas procedurales (sin archivos de imagen) para paredes
audio.rs     → música espacial del chaser + sonido de la habilidad
config.rs    → todas las constantes de balance/tuning en un solo lugar
```

**Regla de dependencia:** `game.rs` es el único que conoce y coordina a todos los demás. Los módulos de dominio (`player`, `chaser`, `map`) no saben nada de rendering ni de `minifb`; reciben `&Map`/`&Player` como parámetros. Esto es buena separación: la lógica de simulación está desacoplada de cómo se dibuja o se lee el input. Si te preguntan "¿por qué separaste así los módulos?", esa es la respuesta.

---

## 3. El corazón del proyecto: raycasting con DDA

Esto es lo que más te van a preguntar. Tenés que poder explicarlo en tu propias palabras, con pizarra si hace falta.

### 3.1 Idea general

Por cada columna de píxeles de la pantalla (960 columnas), se lanza **un rayo** desde la posición del jugador hacia el mundo. El rayo avanza celda por celda hasta chocar con una pared. La **distancia** a la que chocó determina qué tan alta se dibuja esa columna (más cerca → pared más alta/grande).

### 3.2 Plano de cámara y FOV

```rust
let dir_x = player.angle.cos();
let dir_y = player.angle.sin();
let plane_x = -dir_y * FOV_FACTOR;
let plane_y = dir_x * FOV_FACTOR;
```

`dir` es el vector hacia donde mira el jugador. `plane` es perpendicular a `dir` (rotado 90°) y escalado por `FOV_FACTOR` (0.66), que controla el campo de visión. Para cada columna:

```rust
let camera_x = 2.0 * screen_x / width - 1.0;   // va de -1 a 1
let ray_dir = dir + plane * camera_x;
```

`camera_x = -1` es el borde izquierdo de pantalla, `0` es el centro (mirando exactamente hacia `dir`), `1` es el borde derecho. Esto es exactamente la técnica clásica de raycasting tipo Lodev/permadi.

### 3.3 Algoritmo DDA (Digital Differential Analysis)

En vez de avanzar el rayo en pasitos fijos (lento e impreciso), DDA salta **de línea de grilla en línea de grilla**, calculando de antemano cuánto avanza el rayo para cruzar una celda completa en X (`delta_dist_x`) o en Y (`delta_dist_y`):

```rust
delta_dist_x = |1 / ray_dir_x|
delta_dist_y = |1 / ray_dir_y|
```

Y en cada paso del loop, compara `side_dist_x` vs `side_dist_y` (cuánto falta para cruzar el próximo borde vertical u horizontal) y avanza por el que sea más corto. Esto es O(1) por celda cruzada, sin trigonometría cara dentro del loop — muy eficiente comparado con "avanzar de a 0.01 y chequear colisión".

**Por qué importa `side` (0 = eje X, 1 = eje Y):** determina si chocaste con una pared "vertical" u "horizontal" en la grilla, lo cual se usa para:
- Elegir qué componente de textura usar (`wall_x`).
- Aplicar sombreado distinto (`wall_shade_factor`, las paredes en Y se ven un 28% más oscuras — simula luz direccional simple).

### 3.4 Corrección de "ojo de pez" (fisheye)

Si simplemente usaras la distancia euclidiana del rayo, las paredes se verían curvadas (más cerca en los bordes de pantalla que en el centro, aunque sea la misma pared recta). Por eso se calcula la **distancia perpendicular**, no la distancia real del rayo:

```rust
distance = (map_x - pos_x + (1 - step_x)/2) / ray_dir_x   // si side == 0
```

Esto proyecta la distancia sobre el eje de dirección del jugador en vez de sobre la dirección exacta del rayo — es la clave para que las paredes se vean rectas.

### 3.5 De distancia a altura de columna

```rust
line_height = height / distance
draw_start = -line_height/2 + center_y
draw_end   = line_height/2 + center_y
```

Es proyección perspectiva simple: entre más lejos, más chica la columna (relación inversamente proporcional a la distancia).

### 3.6 Texture mapping

`wall_x` (dónde exactamente golpeó el rayo dentro de la celda, 0.0–1.0) se convierte en una columna de textura (`texture_x`, 0–63). Luego, mientras se dibuja la columna verticalmente, se avanza en la textura con un `texture_step = TEXTURE_SIZE / line_height` — esto es interpolación para mapear la altura variable de pantalla a la altura fija de la textura (64 px).

Hay una inversión de textura (`texture_x = TEXTURE_SIZE - texture_x - 1`) según la cara golpeada, para que la textura no salga "espejeada" en paredes que dan al lado contrario.

**Punto clave:** en este proyecto las texturas de pared **no son imágenes**, son funciones procedurales en `texture.rs` (ladrillo, metal, puerta con franjas de peligro, ruinas con grietas) generadas con aritmética de módulo y una función de ruido determinista (hash simple). Sé explicar por qué: evita depender de assets externos para las paredes y es barato de generar. Los *sprites* (el chaser) sí son PNG reales.

---

## 4. Sprites (billboarding)

Los objetos "2D dentro del mundo 3D" (llave, switch, salida, el chaser) se dibujan como sprites que siempre miran de frente a la cámara (billboard).

### 4.1 Transformación mundo → cámara

```rust
inv_det = 1 / (plane_x*dir_y - dir_x*plane_y)
transform_x = inv_det * (dir_y*rel_x - dir_x*rel_y)
transform_y = inv_det * (-plane_y*rel_x + plane_x*rel_y)
```

Esto es invertir la matriz 2x2 `[dir | plane]` para pasar la posición relativa del sprite (`rel_x, rel_y`, en coordenadas de mundo) a coordenadas de cámara. `transform_y` es literalmente la "profundidad" del sprite (qué tan adelante está), y se usa para:
- Descartar sprites detrás de la cámara (`transform_y <= 0.05`).
- Escalar el tamaño en pantalla (`sprite_size ∝ 1/transform_y`) — más lejos, más chico.
- Comparar contra el **depth buffer** para saber si una pared lo tapa.

### 4.2 Depth buffer / z-buffer

`draw_walls` guarda `depth_buffer[screen_x] = hit.distance` por cada columna. Al dibujar sprites, si `transform_y >= depth_buffer[columna]`, esa franja del sprite no se dibuja (la pared está más cerca). Esto resuelve oclusión sin necesidad de ordenar geometría compleja.

### 4.3 Orden pintor (painter's algorithm)

Los sprites se ordenan de **más lejano a más cercano** antes de dibujarlos (`sprites.sort_by(...)` descendente por distancia), para que si dos sprites se superponen, el más cercano quede encima. Como no hay z-buffer *entre sprites* (solo contra paredes), el orden de dibujo es lo que resuelve esa superposición.

---

## 5. Sistema de colisiones

- **Jugador:** se prueban las 4 esquinas de un círculo de radio `PLAYER_RADIUS` (`can_stand_at`) contra el tile de cada esquina — evita "cortar" esquinas de pared en diagonal.
- **Movimiento en sub-pasos:** `move_with_collision` divide un desplazamiento grande en pasos de tamaño `COLLISION_STEP` (0.07) para no "atravesar" paredes delgadas en un solo frame a velocidad alta.
- **Deslizamiento en paredes:** `try_move` mueve X e Y **por separado**, cada uno con su propio chequeo. Si topás una pared de frente pero podés seguir lateralmente, el movimiento lateral sí se aplica — así el jugador "resbala" por la pared en vez de quedar trabado. Es un patrón clásico y barato (no es physically accurate, pero se siente bien).

---

## 6. IA del "chaser" (pared perseguidora)

Tres estados de comportamiento:

1. **Patrulla** (`active == false`): recorre pasillos prefiriendo la celda **menos visitada** (`visit_counts`) entre seguir derecho / girar derecha / girar izquierda, con desempate aleatorio (generador congruencial lineal propio, `advance_rng`, no usa una crate externa de RNG). Esto evita que el patrullaje se vea repetitivo o que se quede atascado en un ciclo corto.
2. **Persecución** (`active == true`): cuando el jugador entra en `CHASER_WAKE_DISTANCE` (6.0), recalcula ruta cada `CHASER_REPATH_INTERVAL` (0.18s) con un **BFS** (`find_path`) sobre la grilla de celdas caminables, y avanza por esa ruta celda a celda. Se desactiva si el jugador se aleja más de `CHASER_LOSE_DISTANCE` (13.0) — la histéresis (wake ≠ lose) evita que oscile activándose/desactivándose en el borde.
3. **Disgusto** (`disgust_timer > 0`): cuando el jugador usa la habilidad sonora (tecla Q) y el chaser está a `SOUND_ABILITY_RANGE` (7.5) o menos, entra en modo repulsión: se aleja del jugador probando 3 direcciones candidatas (directamente en contra, y las dos perpendiculares) por si la dirección directa está bloqueada.

**¿Por qué BFS y no A\*?** Vale la pena poder justificarlo: el mapa es una grilla pequeña (33×25 aprox.) sin costos de movimiento distintos por celda (todo cuesta 1), así que BFS ya encuentra el camino más corto en número de celdas sin necesitar heurística — A* aquí sería complejidad extra sin beneficio real, dado el tamaño del grafo.

---

## 7. Máquina de estados del juego (`GameScreen`)

`game.rs` maneja una máquina de estados explícita con un `enum GameScreen` (MainMenu, LevelSelect, Instructions, About, Playing, LevelSuccess, GameOver). `update_screen` hace un `match` y delega a un `update_*` por pantalla, cada uno devolviendo `bool` (si seguir corriendo el loop o cerrar la ventana). Cada pantalla lee su propio input y llama al método de `Renderer` correspondiente. Es un patrón simple y común para juegos pequeños con varias pantallas — fácil de razonar, aunque no escala infinitamente (si hubiera muchas más pantallas, uno consideraría un patrón más flexible, pero para 7 estados está bien así).

---

## 8. Audio espacial

`rodio::SpatialPlayer` recibe la posición del emisor (chaser) y de dos "oídos" virtuales separados por `EAR_DISTANCE`, en coordenadas relativas al jugador (`right`, `forward`, calculadas con seno/coseno del ángulo). El volumen cae con el cuadrado de la distancia normalizada (`closeness = 1 - ratio²`), y se pausa/ajusta velocidad según si el chaser está activo. La habilidad sonora usa un reproductor separado (`ability_player`) y pausa temporalmente la música del chaser mientras suena.

---

## 9. Preguntas típicas que te pueden hacer (con respuesta corta)

- **¿Qué es DDA y por qué se usa en vez de avanzar el rayo a pasos fijos?**
  Es un algoritmo que salta directo de borde de celda a borde de celda calculando de antemano cuánto avanza el rayo en X y en Y para cruzar una celda completa, evitando revisar colisión en pasos pequeños arbitrarios; es O(número de celdas cruzadas), no O(pasos fijos), y no pierde precisión por el tamaño de paso.

- **¿Por qué se corrige el "fisheye"?**
  Porque la distancia real del rayo depende del ángulo respecto al centro de la pantalla, y eso curva las paredes rectas si se usa directo. Se usa la distancia *perpendicular* al plano de la cámara, no la distancia euclidiana del rayo.

- **¿Cómo se decide el alto de cada columna de pared?**
  Inversamente proporcional a la distancia perpendicular (`height / distance`), simulando perspectiva.

- **¿Cómo funciona el mapeo de texturas si la pared se ve grande o chica en pantalla?**
  Se calcula `texture_step = TEXTURE_SIZE / line_height` y se avanza la coordenada Y de textura por ese paso en cada píxel de pantalla — es un muestreo con paso variable, equivalente a un nearest-neighbor scaling.

- **¿Cómo se dibujan los sprites siempre "de frente" a la cámara (billboarding)?**
  Se transforma su posición relativa al espacio de cámara invirtiendo la matriz `[dir, plane]`, y se calcula tamaño/posición en pantalla en función de esa profundidad transformada; no rotan con su propio eje, solo escalan y se posicionan.

- **¿Cómo se resuelve que una pared tape a un sprite?**
  Comparando la profundidad transformada del sprite contra el `depth_buffer` (distancia de pared) guardado por columna durante `draw_walls`.

- **¿Por qué el jugador se mueve en sub-pasos en vez de aplicar todo el delta de una vez?**
  Para que a velocidades altas o con `dt` grande no "atraviese" paredes delgadas en un solo frame — es una forma barata de mejorar la resolución de colisión sin usar continuous collision detection real.

- **¿Por qué el chaser usa BFS y no A\*?**
  Grilla pequeña, costos uniformes (1 por celda) — BFS ya da camino más corto sin necesitar heurística; A* sería complejidad innecesaria acá.

- **¿Qué hace `dt` (delta time) y por qué es importante?**
  Es el tiempo transcurrido desde el frame anterior (`Instant::now() - previous_frame`). Todo el movimiento se multiplica por `dt` (velocidad × dt) para que el juego se sienta igual sin importar el framerate real — sin esto, el juego iría más rápido en máquinas con más FPS.

- **¿Qué patrón arquitectónico usa `game.rs` para las pantallas?**
  Una máquina de estados explícita con un enum y un `match` que delega a funciones `update_*` por pantalla.

---

## 10. Detalles chiquitos que suenan bien si los mencionás

- Las constantes de balance (velocidades, distancias de detección, cooldowns) están **todas centralizadas en `config.rs`**, no dispersas como "números mágicos" — buena práctica de mantenibilidad, y hay comentarios explicando *por qué* se eligió cada valor (ej. por qué `CHASER_WAKE_DISTANCE` se acortó y `CHASER_LOSE_DISTANCE` se alargó, para efecto de tensión de juego).
- El sistema de audio y de texturas de sprite **fallan de forma segura** (`Option`, con `eprintln!` de aviso) si no encuentra el dispositivo de audio o el archivo de imagen — el juego sigue corriendo sin sonido/con fallback en vez de crashear.
- Las texturas de sprite tratan el color `0xff00ff` (magenta) como transparente (`alpha < 16` se mapea a ese valor) — es la técnica clásica de "color key" en vez de canal alfa real por pixel en el muestreo final.
