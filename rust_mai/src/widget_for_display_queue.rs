use crate::types::*;
use std::collections::BinaryHeap;

struct Data(WidgetForDisplay);

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        self.0.time_stamp_general == other.0.time_stamp_general
    }
}

impl Eq for Data {}

impl PartialOrd for Data {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Data {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.0.time_stamp_general.cmp(&self.0.time_stamp_general)
    }
}

pub struct WidgetForDisplayHeap(BinaryHeap<Data>);

impl WidgetForDisplayHeap {
    pub fn new() -> Self {
        Self(BinaryHeap::new())
    }

    pub fn push(&mut self, widget: WidgetForDisplay) {
        self.0.push(Data(widget));
    }

    pub fn pop(&mut self) -> Option<WidgetForDisplay> {
        self.0.pop().map(|v| v.0)
    }

    pub fn peek(&self) -> Option<&WidgetForDisplay> {
        self.0.peek().map(|v| &v.0)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for WidgetForDisplayHeap {
    fn default() -> Self {
        Self::new()
    }
}
