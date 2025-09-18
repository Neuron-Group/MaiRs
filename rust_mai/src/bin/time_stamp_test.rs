use chrono::Utc;
use tokio::time::{Duration, sleep};

struct TimeS<T: Ord> {
    stamp: T,
}

#[tokio::main]
async fn main() {
    loop {
        let time_stamp_1 = Utc::now();
        let time_stamp_2 = Utc::now();
        let times = TimeS {
            stamp: time_stamp_1,
        };
        assert!(time_stamp_2 > time_stamp_1);
    }
}
