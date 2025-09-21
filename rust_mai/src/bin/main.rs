use chrono::Utc;
use general_time_event_driven::types::{BuildBoxedEventSelector, RuntimeEvent, WorkerMode};
use general_time_event_driven::worker_pool::WorkerPool;
use general_time_event_driven::*;
use macroquad::prelude::*;
use rust_mai::clk::start_clk;
use rust_mai::dev_read::start_key_listen;
use rust_mai::sliding_window::SlidingWindow;
use rust_mai::{parser::*, types::*, widget_for_display_queue::*};
use tokio::sync::mpsc;
use tokio::time::error::Elapsed;
use tokio::time::{Duration, Instant};

use std::collections::HashMap;
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
    // 创建用于排序渲染事件的堆
    let mut sort_heap = WidgetForDisplayHeap::new();

    let block_size = vec2(100.0, 30.0);
    let initial_position_1 = vec2(screen_width() / 4.0 - block_size.x, 50.0);
    let initial_position_2 = vec2(screen_width() / 2.0 - block_size.x, 50.0);
    let initial_position_3 = vec2(screen_width() * 3.0 / 4.0 - block_size.x, 50.0);
    let initial_position_4 = vec2(screen_width() - block_size.x, 50.0);
    let ground_y = screen_height() - 100.0;
    let velocity: f32 = 5000.0;
    let mut sliding_window = SlidingWindow::new();
    // let (tx_rt_event, mut rx_rt_event) = mpsc::channel(200);

    let base_time = Utc::now() + chrono::Duration::seconds(5);
    let (widget_vec, widget_display_vec) = parse_osu_file(
        env!("CARGO_MANIFEST_DIR").to_string() + "/src/bin/test.txt",
        base_time,
        screen_height() as f64 + 1000.,
        velocity.into(),
    );
    let (rt_event_sndr, mut rt_event_rcvr) = tokio::sync::mpsc::channel(10000);
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut hndl_vec = vec![];

            let wkr1_ppty = (
                WkrType::Wkr1,
                WorkerMode::ProcessOnce,
                BuildBoxedEventSelector(|event_tp: &EventType| match event_tp {
                    EventType::D => true,
                    EventType::All => true,
                    _ => false,
                }),
            );
            let wkr2_ppty = (
                WkrType::Wkr2,
                WorkerMode::ProcessOnce,
                BuildBoxedEventSelector(|event_tp: &EventType| match event_tp {
                    EventType::F => true,
                    EventType::All => true,
                    _ => false,
                }),
            );
            let wkr3_ppty = (
                WkrType::Wkr3,
                WorkerMode::ProcessOnce,
                BuildBoxedEventSelector(|event_tp: &EventType| match event_tp {
                    EventType::J => true,
                    EventType::All => true,
                    _ => false,
                }),
            );
            let wkr4_ppty = (
                WkrType::Wkr4,
                WorkerMode::ProcessOnce,
                BuildBoxedEventSelector(|event_tp: &EventType| match event_tp {
                    EventType::K => true,
                    EventType::All => true,
                    _ => false,
                }),
            );
            let wkr0_ppty = (
                WkrType::Wkr0,
                WorkerMode::ProcessMultiTimes,
                BuildBoxedEventSelector(|event_tp: &EventType| true),
            );

            let (event_sndr, wkr_hndl) = WorkerPool::build(
                vec![wkr0_ppty, wkr1_ppty, wkr2_ppty, wkr3_ppty, wkr4_ppty],
                widget_vec,
                rt_event_sndr.clone(),
            )
            .await;

            hndl_vec.push(wkr_hndl.input_worker_handle);
            hndl_vec.push(wkr_hndl.widget_router_handle);

            // let clk_hndl = start_clk(rt_event_sndr, event_sndr).await;
            let (event_mpsc_sndr, mut event_mpsc_rcvr) = tokio::sync::mpsc::channel(5);

            hndl_vec.push(tokio::spawn(async move {
                while let Some(event) = event_mpsc_rcvr.recv().await {
                    event_sndr.send(event).await;
                }
            }));

            hndl_vec.push(start_clk(rt_event_sndr, event_mpsc_sndr.clone()).await);
            hndl_vec.push(start_key_listen(event_mpsc_sndr).await);

            for hndl in hndl_vec {
                hndl.await.unwrap();
            }
        });
    });

    widget_display_vec
        .into_iter()
        .for_each(|e| sort_heap.push(e));

    while !sort_heap.is_empty() {
        sliding_window.push(sort_heap.pop().unwrap());
    }

    loop {
        let return_event = rt_event_rcvr.blocking_recv().unwrap();
        // println!("{return_event:#?}");
        let now = Utc::now();
        // println!("{now}");
        sliding_window
            .end_move_while(|e| e.time_stamp_general - chrono::Duration::seconds(5) < now);
        sliding_window.start_move_while(|e| e.deleted);
        if sliding_window.is_end() {
            break;
        }

        clear_background(WHITE);

        draw_line(0.0, ground_y, screen_width(), ground_y, 3.0, YELLOW);

        if let Some(its) = sliding_window.as_slice() {
            its.iter_mut().for_each(|it| {
                let initial_position_x = match it.place {
                    WkrType::Wkr1 => initial_position_1.x,
                    WkrType::Wkr2 => initial_position_2.x,
                    WkrType::Wkr3 => initial_position_3.x,
                    WkrType::Wkr4 => initial_position_4.x,
                    _ => initial_position_1.x,
                };
                draw_rectangle(
                    initial_position_x,
                    ground_y - (it.time_hit - now).as_seconds_f32() * 1000.,
                    block_size.x,
                    block_size.y,
                    GRAY,
                );
                if let RuntimeEvent::Some(rtv) = &return_event
                    && !rtv.is_blank
                {
                    println!("{rtv:#?}");
                    if it.id == rtv.id && !rtv.is_blank {
                        draw_rectangle(
                            initial_position_x,
                            ground_y - (it.time_hit - now).as_seconds_f32() * 1000.,
                            block_size.x,
                            block_size.y,
                            match rtv.judgement {
                                Judgement::CriticalPerfect => YELLOW,
                                Judgement::Perfect => PINK,
                                Judgement::Good => GREEN,
                            },
                        );
                    } else {
                        draw_rectangle(
                            initial_position_x,
                            ground_y - (it.time_hit - now).as_seconds_f32() * 1000.,
                            block_size.x,
                            block_size.y,
                            GRAY,
                        )
                    }
                }
            });
        }

        next_frame().await;
    }
}
