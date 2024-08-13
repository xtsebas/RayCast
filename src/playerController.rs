use minifb::{Key, Window};
use nalgebra_glm::{Vec2};
use std::f32::consts::PI;
use rodio::{OutputStream, Sink};
use std::io::BufReader;
use std::fs::File;
use crate::{Player, load_maze};

fn is_wall_at(maze: &Vec<Vec<char>>, pos: Vec2, block_size: usize) -> bool {
    let x = pos.x as usize;
    let y = pos.y as usize;

    let i = x / block_size;
    let j = y / block_size;

    // Verificar límites del laberinto
    if i >= maze[0].len() || j >= maze.len() {
        return false;
    }

    // Verificar si la posición está ocupada por una pared
    maze[j][i] == '+' || maze[j][i] == '-' || maze[j][i] == '|'
}

pub fn process_events(window: &Window, player: &mut Player, maze_file: &str) {
    const MOVE_SPEED: f32 = 5.0; // Reducido para movimiento más lento
    const ROTATION_SPEED: f32 = PI / 50.0; // Reducido para rotación más lenta

    // Cargar el laberinto
    let maze = load_maze(maze_file);
    let block_size = 50; // Tamaño de cada bloque en píxeles

    // Obtener la posición del mouse
    if let Some((mouse_x, _mouse_y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
        let mouse_x = mouse_x as f32;
        let delta_x = mouse_x - player.previous_mouse_pos.x;
        if delta_x.abs() > 0.1 {  // Consider a significant movement
            player.a += delta_x.signum() * ROTATION_SPEED;
        }
        player.previous_mouse_pos.x = mouse_x;
    }

    let mut new_pos = player.pos;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let file = File::open("./pasos.mp3").unwrap(); // replace with your sound file path
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    sink.append(source);
    sink.set_volume(10.0); // adjust volume as needed

    // Procesar rotación
    if window.is_key_down(Key::Left) || window.is_key_down(Key::A) {
        player.a -= ROTATION_SPEED; // Girar el punto de vista a la izquierda
    }
    if window.is_key_down(Key::Right) || window.is_key_down(Key::D) {
        player.a += ROTATION_SPEED; // Girar el punto de vista a la derecha
    }

    // Calcular el vector de movimiento
    let move_vec = Vec2::new(player.a.cos(), player.a.sin()) * MOVE_SPEED;

    // Procesar movimiento
    if window.is_key_down(Key::Up) || window.is_key_down(Key::W) {
        new_pos = player.pos + move_vec;
        if !is_wall_at(&maze, new_pos, block_size) {
            player.pos = new_pos;
            sink.play(); // play sound
        }
    }
    if window.is_key_down(Key::Down) || window.is_key_down(Key::S) {
        new_pos = player.pos - move_vec;
        if !is_wall_at(&maze, new_pos, block_size) {
            player.pos = new_pos;
            sink.play(); // play sound
        }
    }
}