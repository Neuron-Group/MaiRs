use std::hash::Hash;

/// 组件Trait
///
/// 定义组件的基本行为，包括工作属性获取和事件判断
pub trait WidgetTrait: Send + Sync {
    type Event: EventTrait;
    fn get_worker_property(&self) -> <<Self as WidgetTrait>::Event as EventTrait>::WorkerProperty;
    fn judge(
        &mut self,
        event: &Self::Event,
    ) -> RuntimeState<<<Self as WidgetTrait>::Event as EventTrait>::ReturnType>;
    fn time_stamp(&self) -> <Self::Event as EventTrait>::TimestampType;
}

impl<Event: EventTrait> PartialEq for dyn WidgetTrait<Event = Event> {
    fn eq(&self, other: &Self) -> bool {
        self.time_stamp() == other.time_stamp()
    }
}

impl<Event: EventTrait> Eq for dyn WidgetTrait<Event = Event> {}

impl<Event: EventTrait> PartialOrd for dyn WidgetTrait<Event = Event> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Event: EventTrait> Ord for dyn WidgetTrait<Event = Event> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time_stamp().cmp(&other.time_stamp())
    }
}

/// 事件Trait
///
/// 定义事件的基本行为，提供事件属性获取功能
pub trait EventTrait: Send + Sync {
    type TimestampType: Ord;
    type EventType: EventTypeTrait;
    type WorkerProperty: WorkerPropertyTrait;
    type ReturnType: ReturnTypeTrait;
    fn get_event_property(&self) -> Self::EventType;
    fn time_stamp(&self) -> Self::TimestampType;
}

impl<
    TimestampType: Ord,
    EventType: EventTypeTrait,
    WorkerProperty: WorkerPropertyTrait,
    ReturnType: ReturnTypeTrait,
> PartialEq
    for dyn EventTrait<
            TimestampType = TimestampType,
            EventType = EventType,
            WorkerProperty = WorkerProperty,
            ReturnType = ReturnType,
        >
{
    fn eq(&self, other: &Self) -> bool {
        self.time_stamp() == other.time_stamp()
    }
}

impl<
    TimestampType: Ord,
    EventType: EventTypeTrait,
    WorkerProperty: WorkerPropertyTrait,
    ReturnType: ReturnTypeTrait,
> Eq
    for dyn EventTrait<
            TimestampType = TimestampType,
            EventType = EventType,
            WorkerProperty = WorkerProperty,
            ReturnType = ReturnType,
        >
{
}

impl<
    TimestampType: Ord,
    EventType: EventTypeTrait,
    WorkerProperty: WorkerPropertyTrait,
    ReturnType: ReturnTypeTrait,
> PartialOrd
    for dyn EventTrait<
            TimestampType = TimestampType,
            EventType = EventType,
            WorkerProperty = WorkerProperty,
            ReturnType = ReturnType,
        >
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<
    TimestampType: Ord,
    EventType: EventTypeTrait,
    WorkerProperty: WorkerPropertyTrait,
    ReturnType: ReturnTypeTrait,
> Ord
    for dyn EventTrait<
            TimestampType = TimestampType,
            EventType = EventType,
            WorkerProperty = WorkerProperty,
            ReturnType = ReturnType,
        >
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time_stamp().cmp(&other.time_stamp())
    }
}

/// 返回值Trait
///
/// 作为通用返回值的标记Trait
pub trait ReturnTypeTrait: Send + Sync {}

/// Box智能指针包装的静态返回值类型
pub type BoxedReturnType<ReturnType> = Box<ReturnType>;

/// 事件判定返回类型
///
/// Pending意味着未完成的判定，组件不会被销毁
/// Ready意味着已完成的判定，组件会被销毁
#[derive(Debug)]
pub enum RuntimeState<ReturnType: ReturnTypeTrait> {
    Pending(RuntimeEvent<ReturnType>),
    Ready(RuntimeEvent<ReturnType>),
}

/// 运行时事件枚举
///
/// 表示处理后的结果事件
#[derive(Debug)]
pub enum RuntimeEvent<ReturnType: ReturnTypeTrait> {
    /// 包含返回值的事件
    Some(ReturnType),
    /// 错过处理的事件
    Missed,
}

/// 事件类型Trait
///
/// 事件类型的标记Trait，用于泛型约束
pub trait EventTypeTrait {}

/// 工作属性Trait
///
/// 工作属性的标记Trait，要求实现基本的哈希和比较功能
pub trait WorkerPropertyTrait: Eq + Hash + Clone + Send + Sync {}

/// 工作模式枚举
///
/// 定义工作线程的事件处理模式
pub enum WorkerMode {
    /// 单次处理模式
    ProcessOnce,
    /// 多次处理模式
    ProcessMultiTimes,
}

/// 制造事件类型判断的闭包
pub fn BuildBoxedEventSelector<
    EventType: EventTypeTrait,
    F: Fn(&EventType) -> bool + Send + Sync + 'static,
>(
    f: F,
) -> Box<dyn Fn(&EventType) -> bool + Send + Sync> {
    let event_selector: Box<dyn Fn(&EventType) -> bool + Send + Sync> = Box::new(f);
    event_selector
}
