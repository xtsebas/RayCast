mod load_maze;
mod framebuffer;
mod bmp;
mod player;
mod playerController;
mod textures;

use load_maze::load_maze;
use textures::Texture;
use framebuffer::Framebuffer;
use player::Player;
use minifb::{Window, WindowOptions, Key};
use std::f32::consts::PI;
use nalgebra_glm::{Vec2};
use std::time::{Duration, Instant};
use playerController::process_events;
use image::GenericImageView;
use std::collections::HashMap;
use image::{RgbImage, RgbaImage};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use rusttype::Scale;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Intersect {
    pub distance: f32,
    pub impact: char
}

pub struct CastRayResult {
    pub intersect: Intersect,
    pub reached_goal: bool,
}

static wall: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture:: new("patterned_plaster_wall_diff_4k.jpg")));
static corner: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture:: new("patterned_plaster_wall_disp_4k.png")));
static jumpscare: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture:: new("creepy.png")));

fn cell_to_texture_color(cell: char, tx: u32, ty: u32) -> u32 {
    let default_color = 0x0000000;

    match cell {
        '+' => corner.get_pixel_color(tx, ty),
        '-' => wall.get_pixel_color(tx, ty),
        '|' => wall.get_pixel_color(tx, ty),
        'g' => 0xFF0000, // Color rojo
        _ => default_color,
    }
}

fn draw_wall_horizontal(framebuffer: &mut Framebuffer, xo: usize, yo: usize, length: usize) {
    framebuffer.set_current_color(0x000000); // Color negro para las paredes
    for x in xo..xo + length {
        framebuffer.point(x, yo);
    }
}

fn draw_wall_vertical(framebuffer: &mut Framebuffer, xo: usize, yo: usize, length: usize) {
    framebuffer.set_current_color(0x000000); // Color negro para las paredes
    for y in yo..yo + length {
        framebuffer.point(xo, y);
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, cell: char, block_size:usize) {
    //let block_size = 50; // Tamaño de cada bloque en píxeles

    match cell {
        '+' => {
            draw_wall_horizontal(framebuffer, xo, yo, block_size);
            draw_wall_vertical(framebuffer, xo, yo, block_size);
        }
        '-' => draw_wall_horizontal(framebuffer, xo, yo, block_size),
        '|' => draw_wall_vertical(framebuffer, xo, yo, block_size),
        'p' => {
            framebuffer.set_current_color(0x00FF00); // Color verde para la meta
            for y in yo..yo + block_size {
                for x in xo..xo + block_size {
                    framebuffer.point(x, y);
                }
            }
        }
        'g' => {
            framebuffer.set_current_color(0xFF0000); // Color rojo para la meta
            for y in yo..yo + block_size {
                for x in xo..xo + block_size {
                    framebuffer.point(x, y);
                }
            }
        }
        ' ' => {
            framebuffer.set_current_color(0xFFFFFF); // Color blanco para el camino
            for y in yo..yo + block_size {
                for x in xo..xo + block_size {
                    framebuffer.point(x, y);
                }
            }
        }
        _ => {}
    }
}

fn render2D(framebuffer: &mut Framebuffer, player: &Player, maze_file: &str) {
    let maze = load_maze(maze_file);
    let block_size = 50; // Tamaño de cada bloque en píxeles

    for (row, line) in maze.iter().enumerate() {
        for (col, &cell) in line.iter().enumerate() {
            draw_cell(framebuffer, col * block_size, row * block_size, cell, 50);
        }
    }

    // Cast a ray from the player's position
    let num_rays = 5;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        cast_ray(framebuffer, &maze, &player, a, block_size, true);
    }
}

fn render3D(framebuffer: &mut Framebuffer, player: &Player, maze_file: &str) {
    let maze = load_maze(maze_file);
    let block_size = 50; // Tamaño de cada bloque en píxeles
    let num_rays = framebuffer.width;

    let hw = framebuffer.width as f32 / 2.0;
    let hh = framebuffer.height as f32 / 2.0;

    // Color blanco para el fondo
    framebuffer.set_current_color(0xFFFFFF);
    framebuffer.clear();

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let intersect = cast_ray(framebuffer, &maze, &player, a, block_size, false);

        let distance_to_wall = intersect.intersect.distance;
        let distance_to_projection_plane = (framebuffer.width as f32 / 2.0) / (player.fov / 2.0).tan();
        
        if distance_to_wall > 0.0 {
            let stake_height = (block_size as f32 * distance_to_projection_plane) / distance_to_wall;
            let stake_top = (hh - (stake_height / 2.0)).max(0.0) as usize;
            let stake_bottom = (hh + (stake_height / 2.0)).min(framebuffer.height as f32) as usize;

            // Estimate texture X coordinate based on ray direction and intersection distance
            let texture_x = ((distance_to_wall / block_size as f32) * wall.width as f32) as u32 % wall.width;

            for y in stake_top..stake_bottom {
                // Calculate texture Y coordinate based on the current height in the column
                let texture_y = ((y - stake_top) as f32 / (stake_bottom - stake_top) as f32 * wall.height as f32) as u32;

                // Determine wall texture color based on cell type
                let wall_color = cell_to_texture_color(intersect.intersect.impact, texture_x, texture_y);

                framebuffer.set_current_color(wall_color);
                if i < framebuffer.width && y < framebuffer.height {
                    framebuffer.point(i, y);
                }
            }

            // Draw ceiling and floor
            if stake_top > 0 {
                framebuffer.set_current_color(0x000000); // Black for the ceiling
                for y in 0..stake_top {
                    if i < framebuffer.width && y < framebuffer.height {
                        framebuffer.point(i, y);
                    }
                }
            }

            if stake_bottom < framebuffer.height as usize {
                framebuffer.set_current_color(0xAAAAAA); // Gray for the floor
                for y in stake_bottom..framebuffer.height as usize {
                    if i < framebuffer.width && y < framebuffer.height {
                        framebuffer.point(i, y);
                    }
                }
            }
        }
    }
}

fn render_fps(framebuffer: &mut Framebuffer, fps: u32) {
    let fps_text = format!("FPS: {}", fps);
    let x = 10; // X position for the text
    let y = 10; // Y position for the text
    let scale = Scale::uniform(20.0); // Font size scale
    let color = 0xFFFFFF; // White color for the text

    framebuffer.drawtext(&fps_text, x, y, scale, color);
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Vec<Vec<char>>,
    player: &Player,
    a: f32,
    block_size: usize,
    draw_line: bool,
) -> CastRayResult {
    let mut d = 0.0;
    let mut hit_wall = false;
    let mut reached_goal = false; // Flag para la meta

    framebuffer.set_current_color(0x0000FF); // Color azul para el rayo

    loop {
        let cos = d * a.cos();
        let sin = d * a.sin();
        let x = (player.pos.x + cos) as usize;
        let y = (player.pos.y + sin) as usize;

        let i = x / block_size;
        let j = y / block_size;

        if i >= maze[0].len() || j >= maze.len() {
            return CastRayResult {
                intersect: Intersect { distance: d, impact: ' ' },
                reached_goal,
            };
        }

        match maze[j][i] {
            '+' | '-' | '|' => {
                hit_wall = true;
                return CastRayResult {
                    intersect: Intersect { distance: d, impact: maze[j][i] },
                    reached_goal,
                };
            }
            'p' | 'g' => {
                if maze[j][i] == 'g' {
                    reached_goal = true;
                }
                return CastRayResult {
                    intersect: Intersect { distance: d, impact: maze[j][i] },
                    reached_goal,
                };
            }
            ' ' => {
                if draw_line {
                    framebuffer.point(x, y);
                }
            }
            _ => {
                return CastRayResult {
                    intersect: Intersect { distance: d, impact: maze[j][i] },
                    reached_goal,
                };
            }
        }

        d += 1.0;
    }
}

fn play_background_music(stream_handle: &rodio::OutputStreamHandle) {
    loop {
        let music_file = File::open("music.mp3").unwrap();
        let music_source = Decoder::new(BufReader::new(music_file)).unwrap();
        
        let music_sink = Sink::try_new(&stream_handle).unwrap();
        music_sink.append(music_source);
        
        // Esperar hasta que la música termine de reproducirse
        music_sink.sleep_until_end();
    }
}

fn render_minimap(framebuffer: &mut Framebuffer, maze: &[Vec<char>], block_size: usize, player: &Player) {
    let maze_width = maze[0].len();
    let maze_height = maze.len();

    // Calcula la escala para ajustar el laberinto al tamaño del minimapa
    let minimap_size = 200; // Tamaño máximo del minimapa en píxeles
    let scale_x = minimap_size as f32 / (maze_width as f32 * block_size as f32);
    let scale_y = minimap_size as f32 / (maze_height as f32 * block_size as f32);
    let scale = scale_x.min(scale_y); // Usamos la menor de las dos escalas para evitar deformaciones

    // Calcula el tamaño dinámico del fondo del minimapa basado en el tamaño del laberinto
    let minimap_width = (maze_width as f32 * block_size as f32 * scale) as usize;
    let minimap_height = (maze_height as f32 * block_size as f32 * scale) as usize;
    let minimap_x = 10; // Posición X del minimapa
    let minimap_y = 10; // Posición Y del minimapa

    // Dibuja el fondo del minimapa
    framebuffer.set_current_color(0x222222); // Fondo oscuro para el minimapa
    for x in minimap_x..(minimap_x + minimap_width) {
        for y in minimap_y..(minimap_y + minimap_height) {
            framebuffer.point(x, y);
        }
    }

    // Dibuja el laberinto en el minimapa
    for row in 0..maze_height {
        for col in 0..maze_width {
            let cell_x = (col as f32 * block_size as f32 * scale) as usize;
            let cell_y = (row as f32 * block_size as f32 * scale) as usize;
            let mini_block_size = (block_size as f32 * scale) as usize;

            // Dibuja cada celda del laberinto en el minimapa
            draw_cell(framebuffer, minimap_x + cell_x, minimap_y + cell_y, maze[row][col], mini_block_size);
        }
    }

    // Dibuja la posición del jugador en el minimapa
    framebuffer.set_current_color(0xFF0000); // Color rojo para el jugador
    let player_x = (player.pos.x * scale) as usize;
    let player_y = (player.pos.y * scale) as usize;
    framebuffer.point(minimap_x + player_x, minimap_y + player_y);
}

fn scale_texture(texture: &Texture, scale: f32) -> Vec<u32> {
    let original_width = texture.width as usize;
    let original_height = texture.height as usize;
    let new_width = (original_width as f32 * scale) as usize;
    let new_height = (original_height as f32 * scale) as usize;

    let mut scaled_texture = vec![0; new_width * new_height];

    for new_y in 0..new_height {
        for new_x in 0..new_width {
            // Calcular las coordenadas de la textura original
            let orig_x = (new_x as f32 / scale) as usize;
            let orig_y = (new_y as f32 / scale) as usize;

            // Obtener el color del píxel de la textura original
            let color = texture.get_pixel_color(orig_x as u32, orig_y as u32);

            // Establecer el color en la textura escalada
            scaled_texture[new_y * new_width + new_x] = color;
        }
    }

    scaled_texture
}

fn render_jumpscare(framebuffer: &mut Framebuffer, player: &Player, block_size: usize) {
    // Tamaño de la textura del enemigo
    let texture_width = jumpscare.width as f32;
    let texture_height = jumpscare.height as f32;

    // Escalar la textura del enemigo (por ejemplo, reducir al 50% del tamaño original)
    let scale = 0.5;
    let scaled_texture = scale_texture(&jumpscare, scale);

    // Tamaño de la textura escalada
    let scaled_width = (texture_width * scale) as i32;
    let scaled_height = (texture_height * scale) as i32;

    // Posición central de la pantalla
    let center_x = framebuffer.width as i32 / 2;
    let center_y = framebuffer.height as i32 / 2;

    // Calcular la posición en el framebuffer para centrar la textura escalada
    let render_x = center_x - scaled_width / 2;
    let render_y = center_y - scaled_height / 2;

    for y in 0..scaled_height {
        for x in 0..scaled_width {
            let color = scaled_texture[(y * scaled_width + x) as usize];
            if color != 0xFFFFFF { // Ignorar el color negro
                let screen_x = render_x + x;
                let screen_y = render_y + y;

                // Verificar que las coordenadas están dentro del framebuffer
                if screen_x >= 0 && screen_x < framebuffer.width as i32 &&
                   screen_y >= 0 && screen_y < framebuffer.height as i32 {
                    framebuffer.set_current_color(color);
                    framebuffer.point(screen_x as usize, screen_y as usize);
                }
            }
        }
    }
}


fn main() {
    let mut maze_file = "maze1.txt"; // Laberinto por defecto

    // Cargar el laberinto y obtener sus dimensiones
    let mut maze = load_maze(maze_file);
    let mut height = maze.len();
    let mut width = maze[0].len();
    let block_size = 50; // Tamaño de cada bloque en píxeles
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let stream_handle = Arc::new(stream_handle); 

    // Crear el framebuffer con las dimensiones adecuadas
    let mut framebuffer = Framebuffer::new(width * block_size, height * block_size);

    let mut welcome_window = Window::new(
        "Bienvenido a Laberinto",
        framebuffer.width,
        framebuffer.height,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("Window creation failed: {}", e);
    });

    let mut welcome_buffer = vec![0; framebuffer.width * framebuffer.height];

    while welcome_window.is_open() && !welcome_window.is_key_down(Key::Enter) {
        let start_time = Instant::now();
        
        // Dibujar un color de fondo
        for i in 0..welcome_buffer.len() {
            welcome_buffer[i] = 0x000000; // Negro
        }
    
        // Dibujar texto en el buffer de bienvenida
        let scale = Scale::uniform(32.0);
        let text = "Bienvenido, Elige el nivel para jugar \n Presiona 1 para nivel 1 \n Presiona 2 para nivel 2 \n Presiona 3 para nivel 3";
        framebuffer.clear();
        framebuffer.drawtext(&text, 10, 10, scale, 0xFFFFFF); // Asegurarse que el color es 0xFFFFFF para blanco

        
    
        // Actualizar el contenido de `welcome_buffer`
        welcome_window.update_with_buffer(&framebuffer.buffer, framebuffer.width, framebuffer.height).unwrap();
        
        // Manejar la entrada del teclado
        if welcome_window.is_key_down(Key::Key1) {
            maze_file = "maze1.txt";
            break; 
        } else if welcome_window.is_key_down(Key::Key2) {
            maze_file = "maze2.txt";
            break; 
        } else if welcome_window.is_key_down(Key::Key3) {
            maze_file = "maze3.txt";
            break;
        }
    }
    
    // Cerrar la ventana de bienvenida y proceder a la ventana principal
    drop(welcome_window);

    // Cargar el laberinto y obtener sus dimensiones
    maze = load_maze(maze_file);
    height = maze.len();
    width = maze[0].len();

    // Variables para controlar el tiempo de aparición del enemigo
    let mut last_jumpscare_spawn = Instant::now();
    let jumpscare_spawn_interval = Duration::new(13, 0);
    let jumpscare_display_time = Duration::new(2, 0);
    let mut show_jumpscare = false;

    // Inicializar el jugador en la posición del carácter 'p'
    let mut player_pos = (0.0, 0.0);
    for (y, row) in maze.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == 'p' {
                player_pos = (x as f32 * block_size as f32, y as f32 * block_size as f32);
            }
        }
    }
    let mut player = Player::new(player_pos.0, player_pos.1, PI / 3.0, PI / 3.0);

    let mut window_game = Window::new(
        "Laberinto - Framebuffer",
        framebuffer.width,
        framebuffer.height,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("Window creation failed: {}", e);
    });

    let mut mode = "3D";
    let mut game_over = false;

    // Frame timing
    let mut last_fps_update = Instant::now();
    let fps_update_interval = Duration::new(1, 0);
    let frame_duration = Duration::new(1, 0) / 60;

    // Crear una variable compartida para controlar el sonido del jumpscare
    let sound_played = Arc::new(Mutex::new(false));

    let stream_handle_clone = Arc::clone(&stream_handle);
    thread::spawn(move || {
        play_background_music(&stream_handle_clone);
    });

    while window_game.is_open() && !window_game.is_key_down(Key::Escape) && !game_over {
        let start_time = Instant::now();

        // Toggle mode
        if window_game.is_key_down(Key::M) {
            mode = if mode == "2D" { "3D" } else { "2D" };
        }

        // Process events
        process_events(&window_game, &mut player, maze_file);

        if !game_over {
            framebuffer.clear();
            if mode == "2D" {
                render2D(&mut framebuffer, &player, maze_file);
            } else {
                let cast_result = cast_ray(&mut framebuffer, &maze, &player, player.a, block_size, false);
                if cast_result.reached_goal {
                    game_over = true;
                }
                render3D(&mut framebuffer, &player, maze_file);
            }

            render_minimap(&mut framebuffer, &maze, block_size, &player);

            // Control del tiempo de aparición del enemigo
            let now = Instant::now();
            if now.duration_since(last_jumpscare_spawn) >= jumpscare_spawn_interval {
                last_jumpscare_spawn = now;
                show_jumpscare = true;
                // Reproducir el sonido del jumpscare en un hilo separado
                let sound_played_clone = Arc::clone(&sound_played);
                let stream_handle_clone = Arc::clone(&stream_handle);
                thread::spawn(move || {
                    let enemy_sound = Decoder::new(File::open("screamer.mp3").unwrap()).unwrap();
                    let mut enemy_sink = Sink::try_new(&*stream_handle_clone).unwrap();
                    enemy_sink.append(enemy_sound);
                    enemy_sink.sleep_until_end();
                    *sound_played_clone.lock().unwrap() = false;
                });
            }

            // Renderizar el enemigo si es el momento adecuado
            if show_jumpscare && now.duration_since(last_jumpscare_spawn) <= jumpscare_display_time {
                render_jumpscare(&mut framebuffer, &player, block_size);
            } else if now.duration_since(last_jumpscare_spawn) > jumpscare_display_time {
                show_jumpscare = false;
            }

            // Calculate FPS and render it
            let elapsed = start_time.elapsed();
            let fps = (1.0 / elapsed.as_secs_f32()).round() as u32;
            render_fps(&mut framebuffer, fps);

            // Update window with framebuffer
            window_game.update_with_buffer(&framebuffer.buffer, framebuffer.width, framebuffer.height).unwrap();
        } else {
            framebuffer.clear();
            window_game.update_with_buffer(&framebuffer.buffer, framebuffer.width, framebuffer.height).unwrap();
        }

        // Esperar hasta el siguiente cuadro para mantener el FPS
        let frame_time = Instant::now().duration_since(start_time);
        if frame_time < frame_duration {
            std::thread::sleep(frame_duration - frame_time);
        }
    }
    drop(window_game);

    let mut screen = Window::new(
        "FELICITACIONES",
        framebuffer.width,
        framebuffer.height,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("Window creation failed: {}", e);
    });

    if game_over {
        while screen.is_open() && !screen.is_key_down(Key::Enter) && !screen.is_key_down(Key::Escape) {
            let start_time = Instant::now();
            
            // Dibujar un color de fondo
            for i in 0..framebuffer.buffer.len() {
                framebuffer.buffer[i] = 0x000000; // Negro
            }
    
            // Dibujar texto en el buffer de bienvenida
            let scale = Scale::uniform(32.0);
            let text = "FELICIDADES, GANASTE";
            framebuffer.drawtext(&text, 10, 10, scale, 0xFFFFFF); // Asegurarse que el color es 0xFFFFFF para blanco
    
            // Actualizar el contenido de `framebuffer`
            screen.update_with_buffer(&framebuffer.buffer, framebuffer.width, framebuffer.height).unwrap();
    
            // Opcional: Manejar la entrada del teclado para cerrar la ventana
            if screen.is_key_down(Key::Enter) || screen.is_key_down(Key::Escape) {
                break; // Salir del bucle si se presiona Enter o Escape
            }
        }
    }
    
    
    // Cerrar la ventana de bienvenida y proceder a la ventana principal
    drop(screen);
}