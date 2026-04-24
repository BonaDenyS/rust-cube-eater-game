use my_game_engine::{
    clear_screen_wrapper, on_key_press, spawn_sprite,
    move_sprite, should_window_close, update_window,
    render_sprite_wrapper, get_time,
    GLFW_KEY_W, GLFW_KEY_A, GLFW_KEY_S, GLFW_KEY_D,
    GLFW_KEY_UP, GLFW_KEY_LEFT, GLFW_KEY_DOWN, GLFW_KEY_RIGHT,
    GLFW_KEY_ESCAPE, GLFW_KEY_R,
    Sprite,
};
use crate::sprite_loader::{fetch_sprite_config, SpriteConfig};
use std::sync::mpsc;

// --- Constants -----------------------------------------------------------

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;
const CELL_SIZE: i32 = 20;
const GRID_W: i32 = WINDOW_WIDTH / CELL_SIZE;   // 40 columns
const GRID_H: i32 = WINDOW_HEIGHT / CELL_SIZE;  // 30 rows
const MOVE_INTERVAL: f64 = 0.12;                // seconds per game step

// --- Direction -----------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// --- GameState -----------------------------------------------------------

pub struct GameState {
    // Snake: list of grid cells (head first)
    snake: Vec<(i32, i32)>,
    dir: Direction,
    next_dir: Direction,

    // Sprites – one per visual role, repositioned each frame
    head_sprite: *mut Sprite,  // red
    body_sprite: *mut Sprite,  // dark green  (reused for every segment)
    food_sprite: *mut Sprite,  // yellow / remote colour

    // Game data
    food_pos: (i32, i32),
    last_update: f64,
    game_over: bool,
    score: u32,

    // HTTP sprite config (multithreaded fetch)
    collectible_rx: mpsc::Receiver<SpriteConfig>,
    collectible_loaded: bool,
}

impl GameState {
    pub fn new() -> Self {
        let head_sprite = spawn_sprite!(
            10 * CELL_SIZE, 15 * CELL_SIZE,
            CELL_SIZE, CELL_SIZE,
            255, 50, 50     // red = head
        );
        let body_sprite = spawn_sprite!(
            0, 0,
            CELL_SIZE, CELL_SIZE,
            0, 180, 0       // dark green = body
        );
        let food_sprite = spawn_sprite!(
            20 * CELL_SIZE, 10 * CELL_SIZE,
            CELL_SIZE, CELL_SIZE,
            255, 220, 0     // yellow = food (may be overridden by server)
        );

        let (tx, rx) = mpsc::channel();
        fetch_sprite_config(tx);   // spawns background thread

        GameState {
            snake: vec![(10, 15), (9, 15), (8, 15)],
            dir: Direction::Right,
            next_dir: Direction::Right,
            head_sprite,
            body_sprite,
            food_sprite,
            food_pos: (20, 10),
            last_update: get_time(),
            game_over: false,
            score: 0,
            collectible_rx: rx,
            collectible_loaded: false,
        }
    }

    // Simple deterministic pseudo-random based on score
    fn next_food_pos(&self) -> (i32, i32) {
        let a = self.score as u64;
        let x = (a.wrapping_mul(1_664_525).wrapping_add(1_013_904_223) % GRID_W as u64) as i32;
        let y = (a.wrapping_mul(22_695_477).wrapping_add(1) % GRID_H as u64) as i32;
        (x, y)
    }

    // Apply colour / position received from the HTTP thread
    fn apply_remote_config(&mut self, cfg: SpriteConfig) {
        let gx = (cfg.x as i32 / CELL_SIZE).clamp(0, GRID_W - 1);
        let gy = (cfg.y as i32 / CELL_SIZE).clamp(0, GRID_H - 1);
        self.food_pos = (gx, gy);
        // Tint the food sprite with server colour
        unsafe {
            (*self.food_sprite).color[0] = cfg.r;
            (*self.food_sprite).color[1] = cfg.g;
            (*self.food_sprite).color[2] = cfg.b;
        }
        self.collectible_loaded = true;
        println!("Food updated from server: {:?}", cfg);
    }

    fn reset(&mut self) {
        self.snake = vec![(10, 15), (9, 15), (8, 15)];
        self.dir = Direction::Right;
        self.next_dir = Direction::Right;
        self.food_pos = (20, 10);
        self.game_over = false;
        self.score = 0;
        self.last_update = get_time();
        // Restore default food colour
        unsafe {
            (*self.food_sprite).color[0] = 255;
            (*self.food_sprite).color[1] = 220;
            (*self.food_sprite).color[2] = 0;
        }
    }

    // --- Input -----------------------------------------------------------

    fn handle_input(&mut self) {
        on_key_press!(GLFW_KEY_W,     { if self.dir != Direction::Down  { self.next_dir = Direction::Up;    } });
        on_key_press!(GLFW_KEY_UP,    { if self.dir != Direction::Down  { self.next_dir = Direction::Up;    } });
        on_key_press!(GLFW_KEY_S,     { if self.dir != Direction::Up    { self.next_dir = Direction::Down;  } });
        on_key_press!(GLFW_KEY_DOWN,  { if self.dir != Direction::Up    { self.next_dir = Direction::Down;  } });
        on_key_press!(GLFW_KEY_A,     { if self.dir != Direction::Right { self.next_dir = Direction::Left;  } });
        on_key_press!(GLFW_KEY_LEFT,  { if self.dir != Direction::Right { self.next_dir = Direction::Left;  } });
        on_key_press!(GLFW_KEY_D,     { if self.dir != Direction::Left  { self.next_dir = Direction::Right; } });
        on_key_press!(GLFW_KEY_RIGHT, { if self.dir != Direction::Left  { self.next_dir = Direction::Right; } });

        if self.game_over {
            on_key_press!(GLFW_KEY_R, { self.reset(); });
        }
    }

    fn update(&mut self) {
        if self.game_over {
            return;
        }

        let now = get_time();
        if now - self.last_update < MOVE_INTERVAL {
            return;
        }
        self.last_update = now;

        self.dir = self.next_dir;
        let (hx, hy) = self.snake[0];
        let new_head = match self.dir {
            Direction::Up    => (hx,     hy - 1),
            Direction::Down  => (hx,     hy + 1),
            Direction::Left  => (hx - 1, hy),
            Direction::Right => (hx + 1, hy),
        };

        // Wall collision
        if new_head.0 < 0 || new_head.0 >= GRID_W
            || new_head.1 < 0 || new_head.1 >= GRID_H
        {
            self.game_over = true;
            println!("Game Over! Score: {} | Press R to restart", self.score);
            return;
        }

        // Self collision
        if self.snake[1..].contains(&new_head) {
            self.game_over = true;
            println!("Game Over! Score: {} | Press R to restart", self.score);
            return;
        }

        self.snake.insert(0, new_head);

        if new_head == self.food_pos {
            // Eat food: grow (keep tail), move food
            self.score += 1;
            println!("Score: {}", self.score);
            self.food_pos = self.next_food_pos();
        } else {
            // Normal move: remove tail
            self.snake.pop();
        }
    }

    // --- Render ----------------------------------------------------------

    fn render(&mut self) {
        clear_screen_wrapper();

        // Food
        move_sprite(
            self.food_sprite,
            (self.food_pos.0 * CELL_SIZE) as f32,
            (self.food_pos.1 * CELL_SIZE) as f32,
        );
        render_sprite_wrapper(self.food_sprite);

        // Body – reuse the single body_sprite, repositioning it per segment
        for &(bx, by) in &self.snake[1..] {
            move_sprite(
                self.body_sprite,
                (bx * CELL_SIZE) as f32,
                (by * CELL_SIZE) as f32,
            );
            render_sprite_wrapper(self.body_sprite);
        }

        // Head (drawn on top of body)
        move_sprite(
            self.head_sprite,
            (self.snake[0].0 * CELL_SIZE) as f32,
            (self.snake[0].1 * CELL_SIZE) as f32,
        );
        render_sprite_wrapper(self.head_sprite);

        update_window();
    }

    // --- Main game loop --------------------------------------------------

    pub fn run(&mut self) {
        while should_window_close() == 0 {
            // Poll for remote sprite config (non-blocking)
            if !self.collectible_loaded {
                if let Ok(cfg) = self.collectible_rx.try_recv() {
                    self.apply_remote_config(cfg);
                }
            }

            on_key_press!(GLFW_KEY_ESCAPE, { break; });

            self.handle_input();
            self.update();
            self.render();
        }
    }
}
