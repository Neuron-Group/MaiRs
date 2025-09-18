#![feature(associated_type_defaults)]

pub mod event_queue;
pub mod types;
pub mod widget_queue;
pub mod worker_pool;

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::{any::Any, sync::Arc};
    use tokio::sync::{broadcast, mpsc};
}
