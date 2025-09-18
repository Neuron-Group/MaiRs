use crate::types::*;
use async_priority_queue::PriorityQueue;
use std::sync::Arc;

struct Data<Event: EventTrait>(Event);

impl<Event: EventTrait> PartialEq for Data<Event> {
    fn eq(&self, other: &Self) -> bool {
        self.0.time_stamp() == other.0.time_stamp()
    }
}

impl<Event: EventTrait> Eq for Data<Event> {}

impl<Event: EventTrait> PartialOrd for Data<Event> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Event: EventTrait> Ord for Data<Event> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.0.time_stamp().cmp(&self.0.time_stamp())
    }
}

struct EventPipe<Event: EventTrait>(PriorityQueue<Data<Event>>);

impl<Event: EventTrait> EventPipe<Event> {
    fn new() -> Self {
        EventPipe::<Event>(PriorityQueue::new())
    }
}

pub struct Sender<Event: EventTrait> {
    socket: Arc<EventPipe<Event>>,
}

pub struct Receiver<Event: EventTrait> {
    socket: Arc<EventPipe<Event>>,
}

pub fn channel<Event: EventTrait>() -> (Sender<Event>, Receiver<Event>) {
    let event_pipe = EventPipe::new();
    let arc_event_pipe = Arc::new(event_pipe);
    (
        Sender {
            socket: arc_event_pipe.clone(),
        },
        Receiver {
            socket: arc_event_pipe,
        },
    )
}

impl<Event: EventTrait> Sender<Event> {
    pub async fn send(&self, event: Event) {
        self.socket.as_ref().0.push(Data(event));
    }
}

impl<Event: EventTrait> Receiver<Event> {
    pub async fn recv(&self) -> Event {
        let Data(result) = self.socket.as_ref().0.pop().await;
        result
    }
}
