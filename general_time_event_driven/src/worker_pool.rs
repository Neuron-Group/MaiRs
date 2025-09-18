use crate::{event_queue, types::*, widget_queue::*};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinHandle,
};

const BUFFER_LENGTH: usize = 10000;

struct WorkerHandle<Event: EventTrait, Widget: WidgetTrait<Event = Event>> {
    widget_sender: mpsc::Sender<Widget>,
    process_handle: JoinHandle<()>,
}

impl<Event: EventTrait + 'static, Widget: WidgetTrait<Event = Event> + 'static>
    WorkerHandle<Event, Widget>
{
    fn new(
        mut event_receiver: broadcast::Receiver<Arc<Widget::Event>>,
        event_worker_mode: WorkerMode,
        runtime_event_sender: mpsc::Sender<
            RuntimeEvent<<<Widget as WidgetTrait>::Event as EventTrait>::ReturnType>,
        >,
        runtime_widget_sender_pre: mpsc::Sender<Widget>,
        event_selector: Box<
            dyn Fn(&<Widget::Event as EventTrait>::EventType) -> bool + Send + Sync,
        >,
    ) -> Self {
        let (widget_sender, mut widget_receiver) = mpsc::channel(BUFFER_LENGTH);
        Self {
            widget_sender,
            process_handle: tokio::spawn(async move {
                let mut widget_heap = WidgetHeap::new();
                while let Ok(event) = event_receiver.recv().await {
                    while !widget_receiver.is_empty() {
                        widget_heap.push(widget_receiver.recv().await.unwrap());
                    }
                    if event_selector(&event.as_ref().get_event_property()) {
                        match event_worker_mode {
                            WorkerMode::ProcessOnce => {
                                if !widget_heap.is_empty()
                                    && let Some(widget) = widget_heap.peek()
                                    && widget.time_stamp() <= event.time_stamp()
                                {
                                    let mut widget = widget_heap.pop().unwrap();
                                    let runtime_state = widget.judge(event.as_ref());
                                    match runtime_state {
                                        RuntimeState::Pending(runtime_event) => {
                                            let _ = runtime_event_sender.send(runtime_event).await;
                                            let _ = runtime_widget_sender_pre.send(widget).await;
                                        }
                                        RuntimeState::Ready(runtime_event) => {
                                            let _ = runtime_event_sender.send(runtime_event).await;
                                        }
                                    }
                                }
                            }
                            WorkerMode::ProcessMultiTimes => {
                                while !widget_heap.is_empty() {
                                    if let Some(widget) = widget_heap.peek()
                                        && widget.time_stamp() <= event.time_stamp()
                                    {
                                        let mut widget = widget_heap.pop().unwrap();
                                        let runtime_state = widget.judge(event.as_ref());
                                        match runtime_state {
                                            RuntimeState::Pending(runtime_event) => {
                                                let _ =
                                                    runtime_event_sender.send(runtime_event).await;
                                                let _ =
                                                    runtime_widget_sender_pre.send(widget).await;
                                            }
                                            RuntimeState::Ready(runtime_event) => {
                                                let _ =
                                                    runtime_event_sender.send(runtime_event).await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }),
        }
    }
}

/// 工作池结构体，管理事件处理和组件路由
///
/// 负责协调事件分发、组件处理和结果返回，内部包含多个工作线程和路由机制
///
/// # 泛型参数
/// * `TimestampType` - 需实现Ord
/// * `EventType` - 事件类型，需实现EventTypeTrait
/// * `WorkerProperty` - 工作属性类型，需实现WorkerPropertyTrait
pub struct WorkerPool<Event: EventTrait, Widget: WidgetTrait<Event = Event>> {
    // 优先队列线程
    input_worker_handle: JoinHandle<()>,
    _event_broadcast_sender: broadcast::Sender<Arc<Event>>,

    // 哈希表路由线程
    widget_router_handle: JoinHandle<()>,

    // 预留
    _runtime_widget_sender_pre: mpsc::Sender<Widget>,
}

impl<Event: EventTrait + 'static, Widget: WidgetTrait<Event = Event> + 'static>
    WorkerPool<Event, Widget>
{
    /// 构建工作池实例
    ///
    /// 创建事件通道、广播器和工作线程，返回发送器、接收器和工作池实例
    ///
    /// # 参数
    /// * `worker_property` - 工作属性列表，包含工作属性、工作模式和事件选择器
    ///
    /// # 返回值
    /// 元组：(事件发送器, 运行时事件接收器, WorkerPool实例)
    pub async fn build(
        worker_property: Vec<(
            Event::WorkerProperty,
            WorkerMode,
            Box<dyn Fn(&Event::EventType) -> bool + Send + Sync>,
        )>,
        widgets: Vec<Widget>,
    ) -> (
        event_queue::Sender<Event>,
        mpsc::Receiver<RuntimeEvent<Event::ReturnType>>,
        Self,
    ) {
        let (event_pipe_sender, event_pipe_receiver) = event_queue::channel();
        let (event_transmit, _) = broadcast::channel(BUFFER_LENGTH);
        let (runtime_event_sender, runtime_event_receiver) = mpsc::channel(BUFFER_LENGTH);
        let (runtime_widget_sender_pre, mut runtime_widget_receiver_pre) =
            mpsc::channel(BUFFER_LENGTH);
        let workers_table = worker_property
            .into_iter()
            .map(|e| {
                (
                    e.0,
                    WorkerHandle::new(
                        event_transmit.subscribe(),
                        e.1,
                        runtime_event_sender.clone(),
                        runtime_widget_sender_pre.clone(),
                        e.2,
                    ),
                )
            })
            .collect::<HashMap<Event::WorkerProperty, WorkerHandle<Event, Widget>>>();

        let event_broadcast_sender = event_transmit.clone();

        let input_worker_handle = tokio::spawn(async move {
            loop {
                let event = event_pipe_receiver.recv().await;
                if event_transmit.send(Arc::new(event)).is_err() {
                    break;
                }
            }
        });

        let widget_router_handle = tokio::spawn(async move {
            loop {
                let widget = runtime_widget_receiver_pre.recv().await;
                if let Some(widget) = widget
                    && let Some(worker_handle) = workers_table.get(&widget.get_worker_property())
                    && worker_handle.widget_sender.send(widget).await.is_err()
                {
                    break;
                }
            }
        });

        for e in widgets.into_iter() {
            runtime_widget_sender_pre.send(e).await.unwrap()
        }

        (
            event_pipe_sender,
            runtime_event_receiver,
            Self {
                input_worker_handle,
                widget_router_handle,
                _event_broadcast_sender: event_broadcast_sender,
                _runtime_widget_sender_pre: runtime_widget_sender_pre,
            },
        )
    }
}
