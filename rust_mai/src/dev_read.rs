use crate::types::*;
use chrono::Utc;
use evdev::{Device, EventType};
use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

// 定义键盘事件类型
#[derive(Debug, Clone)]
pub enum KeyEvent {
    Pressed(u16),
    Released(u16),
}

// 异步键盘监听器
pub struct AsyncKeyboardListener {
    _tx: mpsc::Sender<KeyEvent>, // 保持发送端存活
}

impl AsyncKeyboardListener {
    // 创建监听器并返回事件接收通道
    pub async fn new() -> Result<(Self, mpsc::Receiver<KeyEvent>), Box<dyn Error>> {
        let (tx, rx) = mpsc::channel(32);

        // 获取所有键盘设备的事件流
        let devices = Self::find_keyboard_devices()?;
        let mut tasks = Vec::new();

        for device_path in devices {
            println!("找到键盘设备: {device_path}");

            // 为每个设备创建异步任务
            let device_tx = tx.clone();
            tasks.push(tokio::spawn(async move {
                if let Err(e) = Self::handle_device(&device_path, device_tx).await {
                    eprintln!("设备 {device_path} 处理错误: {e}");
                }
            }));
        }

        // 保持任务运行
        tokio::spawn(async move {
            for i in tasks {
                i.await.unwrap();
            }
        });

        Ok((Self { _tx: tx }, rx))
    }

    // 处理单个设备的事件流
    async fn handle_device(
        device_path: &str,
        tx: mpsc::Sender<KeyEvent>,
    ) -> Result<(), Box<dyn Error>> {
        // 尝试打开设备
        let device = match Device::open(Path::new(device_path)) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                // 尝试提升权限
                if Self::set_device_permissions(device_path).is_ok() {
                    Device::open(Path::new(device_path))?
                } else {
                    return Err("无法提升设备权限".into());
                }
            }
            Err(e) => return Err(e.into()),
        };

        println!("监听键盘: {}", device.name().unwrap_or("未知设备"));

        // 创建事件流
        let mut stream = device.into_event_stream()?;

        // 处理事件流
        while let Ok(event) = stream.next_event().await {
            if event.event_type() == EventType::KEY {
                let code = event.code();

                let key_event = match event.value() {
                    0 => KeyEvent::Released(code),
                    1 => KeyEvent::Pressed(code),
                    _ => continue, // 忽略长按事件
                };

                // 发送事件到通道
                if tx.send(key_event).await.is_err() {
                    break; // 接收端已关闭
                }
            }
        }

        Ok(())
    }

    // 自动查找所有键盘设备
    fn find_keyboard_devices() -> Result<Vec<String>, Box<dyn Error>> {
        let mut devices = Vec::new();

        // 尝试常见设备路径
        for i in 0..10 {
            let path = format!("/dev/input/event{i}");
            if Path::new(&path).exists()
                && let Ok(device) = Device::open(Path::new(&path))
                && device.supported_events().contains(EventType::KEY)
            {
                devices.push(path);
            }
        }

        // 尝试通过 /dev/input/by-path/ 查找
        if let Ok(entries) = fs::read_dir("/dev/input/by-path/") {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(path) = entry.path().canonicalize() {
                    let path_str = path.to_string_lossy().into_owned();
                    if !devices.contains(&path_str)
                        && let Ok(device) = Device::open(&path)
                        && device.supported_events().contains(EventType::KEY)
                    {
                        devices.push(path_str);
                    }
                }
            }
        }

        if devices.is_empty() {
            Err("未找到键盘设备".into())
        } else {
            Ok(devices)
        }
    }

    // 设置设备权限
    fn set_device_permissions(path: &str) -> Result<(), Box<dyn Error>> {
        println!("尝试提升设备权限: {path}");

        // 尝试添加读取权限
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(perms.mode() | 0o444); // 添加读取权限
        fs::set_permissions(path, perms)?;

        Ok(())
    }
}

pub async fn start_key_listen(sndr: tokio::sync::mpsc::Sender<Event>) -> JoinHandle<()> {
    let (_listener, mut rx) = AsyncKeyboardListener::new().await.unwrap();
    tokio::spawn(async move {
        while let Some(key_evnt) = rx.recv().await {
            if let KeyEvent::Pressed(code) = key_evnt {
                match code {
                    32 => sndr
                        .send(Event {
                            time_stamp: Utc::now(),
                            event_ppty: crate::types::EventType::D,
                        })
                        .await
                        .unwrap(),
                    33 => sndr
                        .send(Event {
                            time_stamp: Utc::now(),
                            event_ppty: crate::types::EventType::F,
                        })
                        .await
                        .unwrap(),
                    36 => sndr
                        .send(Event {
                            time_stamp: Utc::now(),
                            event_ppty: crate::types::EventType::J,
                        })
                        .await
                        .unwrap(),
                    37 => sndr
                        .send(Event {
                            time_stamp: Utc::now(),
                            event_ppty: crate::types::EventType::K,
                        })
                        .await
                        .unwrap(),
                    _ => continue,
                }
            }
        }
    })
}
