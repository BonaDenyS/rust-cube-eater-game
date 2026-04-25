use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
pub struct SpriteConfig {
    pub x: f32,
    pub y: f32,
    pub r: i32,
    pub g: i32,
    pub b: i32,
}

impl Default for SpriteConfig {
    fn default() -> Self {
        SpriteConfig { x: 400.0, y: 200.0, r: 255, g: 200, b: 0 }
    }
}

pub fn fetch_sprite_config(tx: mpsc::Sender<SpriteConfig>) {
    thread::spawn(move || {
        let config = load_remote_config().unwrap_or_default();
        let _ = tx.send(config);
    });
}

fn load_remote_config() -> Result<SpriteConfig, Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::get(
        "https://get-random-sprite-data-dan-chiarlones-projects.vercel.app/api/handler",
    )?;
    let json: serde_json::Value = resp.json()?;

    Ok(SpriteConfig {
        x: json["x"].as_f64().unwrap_or(400.0) as f32,
        y: json["y"].as_f64().unwrap_or(200.0) as f32,
        r: json["r"].as_i64().unwrap_or(255) as i32,
        g: json["g"].as_i64().unwrap_or(200) as i32,
        b: json["b"].as_i64().unwrap_or(0) as i32,
    })
}
