mod framebuffer;
mod maze;
mod player;
mod caster;
mod texture;
mod audio;

use gilrs::Gilrs;
use minifb::{Window, WindowOptions, Key};
use nalgebra_glm::Vec2;
use std::f32::consts::PI;
use std::time::{Duration, Instant};
use framebuffer::Framebuffer;
use caster::cast_ray;
use player::{process_events, process_gamepad_events, Player};
use once_cell::sync::Lazy;
use std::sync::Arc;
use texture::Texture;
use audio::AudioPlayer;

// Pantalla de inicio
static SPLASH_TEXTURE: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/start_screen.png")));
// Texturas
static WALL1: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/wall1.png")));
static WALL2: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/wall2.png")));
static WALL4: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/wall4.png")));
static WALL5: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("assets/wall5.png")));

struct Scene {
    maze_data: Vec<Vec<char>>,
    goal: Option<(usize, usize)>
}

fn cell_to_texture_color(cell: char, tx: u32, ty: u32) -> u32 {
    let default_color = 0x805E3C;
    match cell {
        '+' => WALL4.get_pixel_color(tx, ty),
        '-' => WALL1.get_pixel_color(tx, ty),
        '|' => WALL2.get_pixel_color(tx, ty),
        'g' => WALL5.get_pixel_color(tx, ty),
        _ => default_color
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, block_size: usize, cell: char) {
    if cell == ' ' || cell == 'g' { //No mostrar la meta en el mapa
        return;
    }

    framebuffer.set_current_color(0xEEEEEE);
    for x in xo..xo + block_size {
        for y in yo..yo + block_size {
            framebuffer.point(x, y);
        }
    }
}

fn draw_rect(framebuffer: &mut Framebuffer, x: usize, y: usize, width: usize, height: usize, color: u32) {
    framebuffer.set_current_color(color);
    for yi in y..y + height {
        for xi in x..x + width {
            framebuffer.point(xi, yi);
        }
    }
}

fn render_minimap(framebuffer: &mut Framebuffer, player: &Player, maze: &Vec<Vec<char>>) {
    let num_rows = maze.len();
    let num_cols = maze[0].len();

    // Mapa
    let minimap_background_color = 0x202020;
    let minimap_size_x = num_cols * 15;
    let minimap_size_y = num_rows * 15;
    let minimap_x = 10;
    let minimap_y = 5;
    draw_rect(framebuffer, minimap_x, minimap_y, minimap_size_x, minimap_size_y, minimap_background_color);

    let cell_width = minimap_size_x / num_cols;
    let cell_height = minimap_size_y / num_rows;
    for row in 0..num_rows {
        for col in 0..num_cols {
            let screen_x = minimap_x + col * cell_width;
            let screen_y = minimap_y + row * cell_height;
            draw_cell(framebuffer, screen_x, screen_y, cell_width, maze[row][col]);
        }
    }
    // Jugador en el mapa
    let player_x = (player.pos.x / 100.0).round() as usize;
    let player_y = (player.pos.y / 100.0).round() as usize;
    let player_x = player_x.min(num_cols - 1);
    let player_y = player_y.min(num_rows - 1);
    let player_screen_x = minimap_x + player_x * cell_width;
    let player_screen_y = minimap_y + player_y * cell_height;
    framebuffer.set_current_color(0xFF0000);
    framebuffer.draw_circle(player_screen_x, player_screen_y, 3);
}

// Renderizado 3D
fn render3d(framebuffer: &mut Framebuffer, player: &Player, z_buffer: &mut [f32], scene: &Scene) {
    let maze = &scene.maze_data;
    let num_rays = framebuffer.width;
    let block_size = 100.0;
    let half_height = framebuffer.height as f32 / 2.0;
    let fov_half = player.fov / 2.0;

    // Fondo del cielo y piso
    draw_rect(framebuffer, 0, 0, framebuffer.width, framebuffer.height / 2, 0x083863);
    draw_rect(framebuffer, 0, framebuffer.height / 2, framebuffer.width, framebuffer.height / 2, 0x443c33);

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let angle = player.a - fov_half + player.fov * current_ray;
        let intersect = cast_ray(framebuffer, maze, player, angle, block_size as usize, false);

        let distance = intersect.distance * (angle - player.a).cos();
        let stake_height = (framebuffer.height as f32 / distance) * 70.0;
        let stake_top = (half_height - (stake_height / 2.0)) as usize;
        let stake_bottom = (half_height + (stake_height / 2.0)) as usize;

        z_buffer[i] = distance;
        // Renderización
        for y in stake_top..stake_bottom {
            let ty = (((y as f32 - stake_top as f32) / (stake_bottom as f32 - stake_top as f32)) * 128.0)
                .round() as u32;
            let tx = intersect.tx as u32;
            let color = cell_to_texture_color(intersect.impact, tx, ty);
            framebuffer.set_current_color(color);
            framebuffer.point(i, y);
        }
    }
    // Maneja renders adicionales
    render_minimap(framebuffer, player, maze);
}

fn check_victory(player: &Player, scene: &Scene) -> bool {
    if let Some((goal_row, goal_col)) = scene.goal {
        let player_x = (player.pos.x / 100.0).round() as usize;
        let player_y = (player.pos.y / 100.0).round() as usize;

        return player_x == goal_col && player_y == goal_row;
    }
    false
}

fn load_maze_with_goal(file_path: &str) -> (Vec<Vec<char>>, Option<(usize, usize)>) {
    let mut maze = Vec::new();
    let mut goal = None;

    if let Ok(lines) = std::fs::read_to_string(file_path) {
        for (row_idx, line) in lines.lines().enumerate() {
            let mut row = Vec::new();
            for (col_idx, ch) in line.chars().enumerate() {
                if ch == 'g' {
                    goal = Some((row_idx, col_idx));
                }
                row.push(ch);
            }
            maze.push(row);
        }
    }
    (maze, goal)
}

fn render_splash_screen(framebuffer: &mut Framebuffer, splash_texture: &Texture) {
    // Clear the framebuffer
    framebuffer.clear();

    // Draw splash screen
    let img_width = splash_texture.width;
    let img_height = splash_texture.height;
    let fb_width = framebuffer.width as u32;
    let fb_height = framebuffer.height as u32;

    for y in 0..img_height {
        for x in 0..img_width {
            let color = splash_texture.get_pixel_color(x, y);
            if x < fb_width && y < fb_height {
                framebuffer.set_current_color(color);
                framebuffer.point(x as usize, y as usize);
            }
        }
    }
}

fn show_splash_screen(window: &mut Window, framebuffer: &mut Framebuffer, splash_texture: &Texture) {
    let mut space_pressed = false;

    while window.is_open() {
        if window.is_key_down(Key::Space) {
            if !space_pressed {
                break;
            }
            space_pressed = true;
        } else {
            space_pressed = false;
        }

        render_splash_screen(framebuffer, splash_texture);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer.width, framebuffer.height)
            .expect("Failed to update window buffer");
    }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
    let framebuffer_width = 1300;
    let framebuffer_height = 900;
    let target_fps = 15;
    let frame_delay = Duration::from_secs_f32(1.0 / target_fps as f32); //15 fps
    let time_limit = Duration::from_secs(120); // Tiempo límite de 2 minutos

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Escape Maze",
        window_width,
        window_height,
        WindowOptions::default(),
    ).expect("Failed to create window");

    window.set_position(0, 0);
    window.update();

    framebuffer.set_background_color(0x443c33);

    //Pantalla Inicio
    show_splash_screen(&mut window, &mut framebuffer, &SPLASH_TEXTURE);

    let mut player = Player {
        pos: Vec2::new(150.0, 150.0),
        a: PI / 4.0,
        fov: PI / 3.0
    };

    let gilrs = Gilrs::new().expect("Failed to initialize Gilrs");
    let mut gamepad = None;

    let audio_player = AudioPlayer::new("assets/LookWhatYouMadeMeDo.mp3");
    audio_player.play();

    let (maze_data, goal) = load_maze_with_goal("./maze.txt");
    let scene = Scene {
        maze_data,
        goal
    };

    let start_time = Instant::now();
    let mut frame_count = 0;
    let mut last_fps_time = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let current_time = Instant::now();
        let elapsed_time = current_time - start_time;
        if elapsed_time >= time_limit {
            println!("¡Tiempo agotado! \n Game Over");
            break;
        }

        process_events(&window, &mut player);
        // Control
        for (_id, gamepad_info) in gilrs.gamepads() {
            if gamepad_info.is_connected() {
                gamepad = Some(gamepad_info);
                break;
            }
        }
        if let Some(gamepad) = gamepad {
            process_gamepad_events(&gamepad, &mut player);
        }

        // Clear
        if check_victory(&player, &scene) {
            println!("Clear!");
            break;
        }

        framebuffer.clear();

        let mut z_buffer = vec![f32::INFINITY; framebuffer.width];
        render3d(&mut framebuffer, &player, &mut z_buffer, &scene);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .expect("Failed to update window buffer");

        frame_count += 1;

        if current_time.duration_since(last_fps_time).as_secs() >= 1 {
            let fps = frame_count;
            println!("FPS: {}", fps);
            frame_count = 0;
            last_fps_time = current_time;
        }

        std::thread::sleep(frame_delay);
    }
}