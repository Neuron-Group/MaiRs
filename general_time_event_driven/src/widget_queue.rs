use crate::types::*;
use std::collections::BinaryHeap;

struct Data<Widget: WidgetTrait>(Widget);

impl<Widget: WidgetTrait> PartialEq for Data<Widget> {
    fn eq(&self, other: &Self) -> bool {
        self.0.time_stamp() == other.0.time_stamp()
    }
}

impl<Widget: WidgetTrait> Eq for Data<Widget> {}

impl<Widget: WidgetTrait> PartialOrd for Data<Widget> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Widget: WidgetTrait> Ord for Data<Widget> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.0.time_stamp().cmp(&self.0.time_stamp())
    }
}

pub struct WidgetHeap<Widget: WidgetTrait>(BinaryHeap<Data<Widget>>);

impl<Widget: WidgetTrait> WidgetHeap<Widget> {
    pub fn new() -> Self {
        Self(BinaryHeap::new())
    }

    pub fn push(&mut self, widget: Widget) {
        self.0.push(Data(widget));
    }

    pub fn pop(&mut self) -> Option<Widget> {
        self.0.pop().map(|v| v.0)
    }

    pub fn peek(&self) -> Option<&Widget> {
        self.0.peek().map(|v| &v.0)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<Widget: WidgetTrait> Default for WidgetHeap<Widget> {
    fn default() -> Self {
        Self::new()
    }
}
