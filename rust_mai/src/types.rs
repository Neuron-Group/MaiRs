use chrono::{DateTime, Duration, Utc};
use general_time_event_driven::types::*;

// 事件类型模块
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    OnlyWkr0,
    D,
    F,
    J,
    K,
    All,
}

// Wkr类型模块
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum WkrType {
    Wkr0,
    Wkr1,
    Wkr2,
    Wkr3,
    Wkr4,
}

impl EventTypeTrait for EventType {}

// Wkr 类型模块，直接复用事件类型模块
impl WorkerPropertyTrait for WkrType {}

// 返回类型枚举
#[derive(Debug, Hash)]
pub enum Judgement {
    CriticalPerfect,
    Perfect,
    Good,
}

// 返回值模块
#[derive(Debug, Hash)]
pub struct RtV {
    pub is_blank: bool,
    pub id: usize,
    pub judgement: Judgement,
}

impl ReturnTypeTrait for RtV {}

// 事件模块
#[derive(Debug)]
pub struct Event {
    pub time_stamp: DateTime<Utc>,
    pub event_ppty: EventType,
}

impl EventTrait for Event {
    type TimestampType = DateTime<Utc>;
    type EventType = EventType;
    type WorkerProperty = WkrType;
    type ReturnType = RtV;

    fn get_event_property(&self) -> Self::EventType {
        self.event_ppty
    }

    fn time_stamp(&self) -> Self::TimestampType {
        self.time_stamp
    }
}

// 判定组件模块
#[derive(Debug)]
pub struct Widget {
    pub id: usize,
    pub time_stamp: DateTime<Utc>,
    pub wkr_ppty: WkrType,
}

impl WidgetTrait for Widget {
    type Event = Event;

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
        // 计算时间差（使用Duration）
        let delta = event.time_stamp - self.time_stamp;

        // 应用偏移量（原逻辑中的+20ms）
        let relative_time = delta + Duration::milliseconds(20);

        // 定义判定范围
        let critical_perfect_range = Duration::milliseconds(-1)..=Duration::milliseconds(1);
        let perfect_range = Duration::milliseconds(-5)..=Duration::milliseconds(5);
        let good_range = Duration::milliseconds(-20)..=Duration::milliseconds(20);

        // 进行判定
        if critical_perfect_range.contains(&relative_time) {
            RuntimeState::Ready(RuntimeEvent::Some(RtV {
                is_blank: false,
                id: self.id,
                judgement: Judgement::CriticalPerfect,
            }))
        } else if perfect_range.contains(&relative_time) {
            RuntimeState::Ready(RuntimeEvent::Some(RtV {
                is_blank: false,
                id: self.id,
                judgement: Judgement::Perfect,
            }))
        } else if good_range.contains(&relative_time) {
            RuntimeState::Ready(RuntimeEvent::Some(RtV {
                is_blank: false,
                id: self.id,
                judgement: Judgement::Good,
            }))
        } else {
            RuntimeState::Ready(RuntimeEvent::Missed)
        }
    }
}

// 渲染组件模块
#[derive(Debug)]
pub struct WidgetForDisplay {
    pub id: usize,
    pub time_stamp_general: DateTime<Utc>,
    pub time_stamp_display: DateTime<Utc>,
    pub time_hit: DateTime<Utc>,
    pub place: WkrType,
    pub deleted: bool,
}
