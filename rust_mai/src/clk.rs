use chrono::Utc;
use tokio::{task::JoinHandle, time::Duration};

use crate::types::{Event, RtV};

const FPS: f32 = 120.0;
const EVENT_FRAC: u8 = 10;

pub async fn start_clk(
    sndr_playtrd: tokio::sync::mpsc::Sender<RtV>,
    sndr_eventtrd: tokio::sync::mpsc::Sender<Event>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut cnt: u8 = 0;
        loop {
            cnt += 1;
            cnt %= EVENT_FRAC;
            match cnt {
                0 => sndr_eventtrd
                    .send(Event {
                        time_stamp: Utc::now(),
                        event_ppty: crate::types::EventType::All,
                    })
                    .await
                    .unwrap(),
                _ => sndr_playtrd
                    .send(RtV {
                        is_blank: true,
                        id: 0,
                        judgement: crate::types::Judgement::Good,
                    })
                    .await
                    .unwrap(),
            }

            tokio::time::sleep(Duration::from_secs_f32(1.0 / FPS)).await;
        }
    })
}
