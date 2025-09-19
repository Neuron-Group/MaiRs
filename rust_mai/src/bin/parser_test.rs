use chrono::{DateTime, Duration, Utc};
use rust_mai::{
    parser::parse_osu_file,
    types::{Widget, WidgetForDisplay, WkrType},
};

#[tokio::main]
async fn main() {
    let (widgets, widgets_for_display) = parse_osu_file(
        "/home/neuron/Projects/rust/RustMai/rust_mai/src/bin/test.txt",
        Utc::now(),
        1200.,
        20.,
    );

    println!("解析出的Widgets:");
    for widget in &widgets {
        println!("{:?}", widget);
    }

    println!("\n解析出的WidgetsForDisplay:");
    for widget in &widgets_for_display {
        println!("{:?}", widget);
    }
}
