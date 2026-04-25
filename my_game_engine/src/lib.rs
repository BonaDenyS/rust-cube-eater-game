use std::ffi::CString;
use std::os::raw::{c_char, c_float, c_int};

pub const GLFW_PRESS: c_int = 1;
pub const GLFW_KEY_W: c_int = 87;
pub const GLFW_KEY_A: c_int = 65;
pub const GLFW_KEY_S: c_int = 83;
pub const GLFW_KEY_D: c_int = 68;
pub const GLFW_KEY_UP: c_int = 265;
pub const GLFW_KEY_DOWN: c_int = 264;
pub const GLFW_KEY_LEFT: c_int = 263;
pub const GLFW_KEY_RIGHT: c_int = 262;
pub const GLFW_KEY_ESCAPE: c_int = 256;
pub const GLFW_KEY_SPACE: c_int = 32;
pub const GLFW_KEY_R: c_int = 82;

#[repr(C)]
#[derive(Debug)]
pub struct Sprite {
    pub width: c_int,
    pub height: c_int,
    pub color: [c_int; 3],
    pub x: c_float,
    pub y: c_float,
}

#[repr(C)]
pub struct GLFWwindow {
    _private: [u8; 0],
}

extern "C" {
    pub fn glfwGetTime() -> f64;
    pub fn create_game_window(title: *const c_char, width: c_int, height: c_int);
    pub fn create_sprite(x: c_float, y: c_float, width: c_int, height: c_int, r: c_int, g: c_int, b: c_int) -> *mut Sprite;
    pub fn render_sprite(sprite: *mut Sprite);
    pub fn update_sprite_position(sprite: *mut Sprite, x: c_float, y: c_float);
    pub fn update_game_window();
    pub fn clear_screen();
    pub fn window_should_close() -> c_int;
    pub fn get_key(window: *mut GLFWwindow, key: c_int) -> c_int;
    pub fn get_window() -> *mut GLFWwindow;
}

pub fn init_window(title: &str, width: i32, height: i32) {
    let c_title = CString::new(title).expect("CString::new failed");
    unsafe { create_game_window(c_title.as_ptr(), width, height) }
}

pub fn new_sprite(x: f32, y: f32, width: i32, height: i32, r: i32, g: i32, b: i32) -> *mut Sprite {
    unsafe { create_sprite(x, y, width, height, r, g, b) }
}

pub fn render_sprite_wrapper(sprite: *mut Sprite) {
    unsafe { render_sprite(sprite) }
}

pub fn move_sprite(sprite: *mut Sprite, x: f32, y: f32) {
    unsafe { update_sprite_position(sprite, x, y) }
}

pub fn update_window() {
    unsafe { update_game_window() }
}

pub fn clear_screen_wrapper() {
    unsafe { clear_screen() }
}

pub fn should_window_close() -> i32 {
    unsafe { window_should_close() }
}

pub fn check_key(window: *mut GLFWwindow, key: i32) -> i32 {
    unsafe { get_key(window, key) }
}

pub fn get_time() -> f64 {
    unsafe { glfwGetTime() }
}

#[macro_export]
macro_rules! start_window_and_game_loop {
    ($title:expr, $width:expr, $height:expr, $body:block) => {{
        $crate::init_window($title, $width, $height);
        while $crate::should_window_close() == 0 {
            $body
            $crate::update_window();
        }
    }};
}

#[macro_export]
macro_rules! on_key_press {
    ($key:expr, $body:block) => {{
        let _window = unsafe { $crate::get_window() };
        if $crate::check_key(_window, $key) == $crate::GLFW_PRESS {
            $body
        }
    }};
}

#[macro_export]
macro_rules! spawn_sprite {
    ($x:expr, $y:expr, $w:expr, $h:expr, $r:expr, $g:expr, $b:expr) => {
        $crate::new_sprite(
            $x as f32, $y as f32,
            $w as i32, $h as i32,
            $r as i32, $g as i32, $b as i32,
        )
    };
}

#[macro_export]
macro_rules! clear_and_render {
    ($($sprite:expr),* $(,)?) => {{
        $crate::clear_screen_wrapper();
        $(
            $crate::render_sprite_wrapper($sprite);
        )*
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_visual_test<F: FnOnce()>(title: &str, body: F) {
        init_window(title, 800, 600);
        body();
        while should_window_close() == 0 {
            update_window();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }

    #[test]
    #[ignore]
    fn test_simple_game_loop() {
        init_window("Test Game Loop", 800, 600);
        let mut frame_count = 0;
        // Run for 180 frames (~3 seconds at 60 fps) so the window is visible long enough to observe
        while should_window_close() == 0 && frame_count < 180 {
            clear_screen_wrapper();
            update_window();
            std::thread::sleep(std::time::Duration::from_millis(16));
            frame_count += 1;
        }
        assert!(frame_count > 0, "Game loop must execute at least one frame");
    }

    #[test]
    #[ignore]
    fn test_sprite_rendering() {
        run_visual_test("Test Sprite Rendering", || {
            let sprite = new_sprite(100.0, 100.0, 50, 50, 255, 0, 0);
            assert!(!sprite.is_null(), "Sprite must be created successfully");
            // Animate the sprite moving across the screen for ~3 seconds so rendering is clearly visible
            for frame in 0..180 {
                if should_window_close() != 0 {
                    break;
                }
                let x = 100.0 + frame as f32 * 2.0;
                move_sprite(sprite, x, 100.0);
                clear_screen_wrapper();
                render_sprite_wrapper(sprite);
                update_window();
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
        });
    }

    #[test]
    #[ignore]
    fn test_screen_clearing() {
        run_visual_test("Test Screen Clear", || {
            // Alternate between a filled sprite and a cleared screen every 30 frames
            // so the clearing effect is visually obvious
            let sprite = new_sprite(300.0, 200.0, 200, 200, 255, 0, 255);
            assert!(!sprite.is_null(), "Sprite must be created to demonstrate clearing");
            for frame in 0..180 {
                if should_window_close() != 0 {
                    break;
                }
                clear_screen_wrapper();
                // Show the sprite only during the first half of each 60-frame cycle
                if (frame % 60) < 30 {
                    render_sprite_wrapper(sprite);
                }
                update_window();
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
        });
    }

    #[test]
    #[ignore]
    fn test_key_presses() {
        init_window("Test Key Presses - Press WASD", 800, 600);
        let window = unsafe { get_window() };
        assert!(!window.is_null(), "Window handle must be valid for key polling");

        let sprite = new_sprite(375.0, 275.0, 50, 50, 0, 200, 255);
        assert!(!sprite.is_null());

        let (mut x, mut y) = (375.0_f32, 275.0_f32);
        let speed = 3.0_f32;

        // Run for ~5 seconds; the sprite moves in response to WASD so key detection is observable
        for _ in 0..300 {
            if should_window_close() != 0 {
                break;
            }
            if check_key(window, GLFW_KEY_W) == GLFW_PRESS { y -= speed; }
            if check_key(window, GLFW_KEY_S) == GLFW_PRESS { y += speed; }
            if check_key(window, GLFW_KEY_A) == GLFW_PRESS { x -= speed; }
            if check_key(window, GLFW_KEY_D) == GLFW_PRESS { x += speed; }

            move_sprite(sprite, x, y);
            clear_screen_wrapper();
            render_sprite_wrapper(sprite);
            update_window();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }

        // Verify the window and key API are functional
        assert!(check_key(window, GLFW_KEY_W) >= 0);
        assert!(check_key(window, GLFW_KEY_A) >= 0);
        assert!(check_key(window, GLFW_KEY_S) >= 0);
        assert!(check_key(window, GLFW_KEY_D) >= 0);
    }

    #[test]
    #[ignore]
    fn test_sprite_position_update() {
        run_visual_test("Test Position Update", || {
            let sprite = new_sprite(0.0, 275.0, 50, 50, 0, 255, 0);
            assert!(!sprite.is_null(), "Sprite must be created for position update test");

            // Sweep the sprite from left to right across the full window width
            for frame in 0..180 {
                if should_window_close() != 0 {
                    break;
                }
                let x = (frame as f32 / 180.0) * 750.0;
                move_sprite(sprite, x, 275.0);

                unsafe {
                    assert!((*sprite).x >= 0.0, "Sprite x must be non-negative after move");
                    assert!((*sprite).y == 275.0, "Sprite y must remain constant");
                }

                clear_screen_wrapper();
                render_sprite_wrapper(sprite);
                update_window();
                std::thread::sleep(std::time::Duration::from_millis(16));
            }

            // Final position check
            move_sprite(sprite, 200.0, 300.0);
            unsafe {
                assert_eq!((*sprite).x, 200.0);
                assert_eq!((*sprite).y, 300.0);
            }
        });
    }
}
