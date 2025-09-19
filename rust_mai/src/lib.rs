#![feature(allocator_api)]

pub mod clk;
pub mod dev_read;
pub mod parser;
pub mod sliding_window;
pub mod types;
pub mod widget_for_display_queue;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
