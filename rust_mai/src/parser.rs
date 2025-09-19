use crate::types::{Widget, WidgetForDisplay, WkrType};
use chrono::{DateTime, Duration, Utc};

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

// 解析osu文件并生成组件
pub fn parse_osu_file<P: AsRef<Path>>(
    file_path: P,
    base_time: DateTime<Utc>,
    screen_height: f64,
    scroll_speed: f64,
) -> (Vec<Widget>, Vec<WidgetForDisplay>) {
    let file = File::open(file_path).expect("无法打开文件");
    let reader = BufReader::new(file);

    let mut widgets = Vec::new();
    let mut widgets_for_display = Vec::new();
    let mut id_counter = 0;

    // 假设的屏幕高度和流速（这些值可能需要根据实际情况调整）
    // let screen_height = 384.0; // 像素
    // let scroll_speed = 1.0; // 流速乘数

    // 基准时间（1970年1月1日）
    // let base_time = Utc.ymd(1970, 1, 1).and_hms(0, 0, 0);
    // let base_time = Utc::now();

    let mut in_hit_objects = false;

    for line in reader.lines() {
        let line = line.expect("读取行失败");

        // 跳过注释和空行
        if line.trim().is_empty() || line.trim().starts_with("//") {
            continue;
        }

        // 检查是否进入HitObjects部分
        if line.trim() == "[HitObjects]" {
            in_hit_objects = true;
            continue;
        }

        // 如果不在HitObjects部分，继续读取
        if !in_hit_objects {
            continue;
        }

        // 解析HitObject行
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 4 {
            continue;
        }

        let x = parts[0].parse::<i32>().unwrap_or(0);
        let time_ms = parts[2].parse::<i64>().unwrap_or(0);
        let obj_type = parts[3].parse::<i32>().unwrap_or(0);

        // 只处理普通点击和长条的起点（忽略长条的中间部分）
        if obj_type & 1 == 1 || obj_type & 128 == 128 {
            // 将x坐标映射到WkrType
            let wkr_type = match x {
                64 => WkrType::Wkr1,
                192 => WkrType::Wkr2,
                320 => WkrType::Wkr3,
                448 => WkrType::Wkr4,
                _ => continue, // 跳过不在预期位置的音符
            };

            // 计算时间戳
            let hit_time = Duration::milliseconds(time_ms);
            let widget_time = base_time + hit_time - Duration::milliseconds(1000); // 提前20ms

            // 计算显示时间（基于流速和屏幕高度）
            // 这里简化计算，实际可能需要更复杂的公式
            let approach_time_ms = screen_height / scroll_speed; // 假设AR为5对应的接近时间
            let display_time =
                base_time + hit_time - Duration::milliseconds(approach_time_ms as i64);

            // 创建Widget
            let widget = Widget {
                id: id_counter,
                time_stamp: widget_time,
                wkr_ppty: wkr_type,
            };

            // 创建WidgetForDisplay
            let widget_display = WidgetForDisplay {
                id: id_counter,
                time_stamp_general: if widget_time < display_time {
                    widget_time
                } else {
                    display_time
                },
                time_stamp_display: display_time,
                time_hit: base_time + hit_time,
                place: wkr_type,
                deleted: false,
            };

            widgets.push(widget);
            widgets_for_display.push(widget_display);

            id_counter += 1;
        }
    }

    (widgets, widgets_for_display)
}
