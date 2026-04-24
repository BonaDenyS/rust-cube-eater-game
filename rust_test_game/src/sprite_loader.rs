use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
pub struct SpriteConfig {
    pub x: f32,
    pub y: f32,
    pub width: i32,
    pub height: i32,
    pub r: i32,
    pub g: i32,
    pub b: i32,
}

impl Default for SpriteConfig {
    fn default() -> Self {
        SpriteConfig { x: 400.0, y: 200.0, width: 40, height: 40, r: 255, g: 200, b: 0 }
    }
}

pub fn fetch_sprite_config(tx: mpsc::Sender<SpriteConfig>) {
    thread::spawn(move || {
        let config = load_remote_config().unwrap_or_default();
        let _ = tx.send(config);
    });
}

fn load_remote_config() -> Result<SpriteConfig, Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::get("https://jsonplaceholder.typicode.com/todos/1")?;
    let json: serde_json::Value = resp.json()?;

    let id = json["id"].as_i64().unwrap_or(1) as f32;
    let user_id = json["userId"].as_i64().unwrap_or(1);

    Ok(SpriteConfig {
        x: 50.0 + id * 10.0,
        y: 150.0 + id * 5.0,
        width: 40,
        height: 40,
        r: ((user_id * 60) % 256) as i32,
        g: 200,
        b: ((user_id * 40 + 100) % 256) as i32,
    })
}
