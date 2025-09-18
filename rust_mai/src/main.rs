use general_time_event_driven::*;
use macroquad::prelude::*;
use tokio::sync::mpsc;
use tokio::time::error::Elapsed;
use tokio::time::{Duration, Instant};

use std::thread;

use tokio::runtime;

struct FallingBlock {
    position: Vec2,
    size: Vec2,
    color: Color,
}

struct GroundLine {
    y: f32,
    color: Color,
}

#[macroquad::main("Falling Block With Tokio Timer")]
async fn main() {
    let block_size = vec2(100.0, 30.0);
    let initial_position = vec2(screen_width() / 2.0 - block_size.x, 50.0);
    let ground_y = screen_height() - 100.0;
    let velocity = 5000.0;

    let mut block = FallingBlock {
        position: initial_position,
        size: block_size,
        color: RED,
    };

    let ground = GroundLine {
        y: ground_y,
        color: GREEN,
    };

    let (tx, mut rx) = mpsc::channel(200);

    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let start_time = tokio::time::Instant::now();

            loop {
                let elapsed = start_time.elapsed().as_secs_f32();
                let relative_length = velocity * elapsed;

                if tx.send(relative_length).await.is_err() {
                    break;
                }

                tokio::time::sleep(Duration::from_secs_f32(1.0 / 240.0)).await;
            }
        });
    });

    loop {
        let relative_length = rx.blocking_recv().unwrap();
        block.position.y = initial_position.y + relative_length;

        clear_background(WHITE);

        draw_line(0.0, ground.y, screen_width(), ground.y, 3.0, ground.color);

        draw_rectangle(
            block.position.x,
            block.position.y,
            block.size.x,
            block.size.y,
            block.color,
        );

        next_frame().await;
    }
}
