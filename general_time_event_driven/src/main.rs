use tokio::time::{Duration, sleep};

use general_time_event_driven::{types::*, worker_pool::*};
use std::{any::Any, sync::Arc};
use tokio::sync::{broadcast, mpsc};

type TimeStamp = i64;

// 事件类型模块
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
enum TestEventType {
    Wkr0,
    All,
}

impl EventTypeTrait for TestEventType {}

// Wkr类型模块
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
enum TestWorkerType {
    Wkr0,
    Wkr1,
}

impl WorkerPropertyTrait for TestWorkerType {}

// 返回类型枚举
#[derive(Debug)]
enum Judgement {
    CriticalPerfect,
    Perfect,
    Good,
}

// 返回值模块
#[derive(Debug)]
struct TestRtV {
    id: usize,
    judgement: Judgement,
}

impl ReturnTypeTrait for TestRtV {}

// 事件模块
struct TestEvent {
    time_stamp: TimeStamp,
    event_ppty: TestEventType,
}

impl EventTrait for TestEvent {
    type TimestampType = TimeStamp;
    type EventType = TestEventType;
    type WorkerProperty = TestWorkerType;
    type ReturnType = TestRtV;
    fn get_event_property(&self) -> Self::EventType {
        self.event_ppty
    }
    fn time_stamp(&self) -> Self::TimestampType {
        self.time_stamp
    }
}

// 组件模块
#[derive(Debug)]
struct TestWidget {
    id: usize,
    time_stamp: TimeStamp,
    wkr_ppty: TestWorkerType,
}

impl WidgetTrait for TestWidget {
    type Event = TestEvent;
    fn time_stamp(&self) -> <Self::Event as EventTrait>::TimestampType {
        self.time_stamp
    }
    fn get_worker_property(&self) -> <<Self as WidgetTrait>::Event as EventTrait>::WorkerProperty {
        self.wkr_ppty
    }
    fn judge(
        &mut self,
        event: &Self::Event,
    ) -> RuntimeState<<<Self as WidgetTrait>::Event as EventTrait>::ReturnType> {
        let relative_time = self.time_stamp - event.time_stamp + 20;
        if (-1..=1).contains(&relative_time) {
            RuntimeState::Ready(RuntimeEvent::Some(TestRtV {
                id: self.id,
                judgement: Judgement::CriticalPerfect,
            }))
        } else if (-5..5).contains(&relative_time) {
            RuntimeState::Ready(RuntimeEvent::Some(TestRtV {
                id: self.id,
                judgement: Judgement::Perfect,
            }))
        } else if (-20..=20).contains(&relative_time) {
            RuntimeState::Ready(RuntimeEvent::Some(TestRtV {
                id: self.id,
                judgement: Judgement::Good,
            }))
        } else {
            RuntimeState::Ready(RuntimeEvent::Missed)
        }
    }
}

fn test_judge_perfect() {
    let mut widget = TestWidget {
        id: 12345,
        time_stamp: 1000,
        wkr_ppty: TestWorkerType::Wkr0,
    };
    let event = TestEvent {
        time_stamp: 1024,
        event_ppty: TestEventType::Wkr0,
    };
    println!("{:#?}", widget.judge(&event));
}

async fn test_worker_pool_process() {
    let event_select = BuildBoxedEventSelector(|event_tp: &TestEventType| match event_tp {
        TestEventType::Wkr0 => true,
        TestEventType::All => false,
    });
    let wrk_ppty = vec![(TestWorkerType::Wkr0, WorkerMode::ProcessOnce, event_select)];
    let widget = TestWidget {
        id: 12345,
        time_stamp: 1000,
        wkr_ppty: TestWorkerType::Wkr0,
    };
    let widget_list = vec![widget];
    let (sndr, mut rcvr, hndl) = WorkerPool::build(wrk_ppty, widget_list).await;

    let event = TestEvent {
        time_stamp: 1024,
        event_ppty: TestEventType::Wkr0,
    };
    sndr.send(event).await;
    if let Some(rt_item) = rcvr.recv().await {
        dbg!(rt_item);
    }
}

#[tokio::main]
async fn main() {
    test_judge_perfect();
    test_worker_pool_process().await;
}
