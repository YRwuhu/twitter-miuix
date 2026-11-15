// 桌面壳层入口。
//
// 窗口始终保持手机竖屏比例 9:19.5（432×936 的逻辑基准），并根据当前显示器的
// 可用尺寸等比缩放，避免在高 DPI 的小屏上底部导航被挤出屏幕。
// 后续网络、缓存与凭据存储都应放在 Rust 侧。
use std::time::Duration;
use tauri::{LogicalSize, Manager, Size, WebviewWindow};

mod commands;
mod dm;
mod secrets;
mod models;
mod query_ids;
mod x_api;

/// 432 / 936 = 9 / 19.5。
const PHONE_ASPECT: f64 = 432.0 / 936.0;
const BASE_HEIGHT: f64 = 936.0;
/// 标题栏 + 任务栏预留的逻辑像素，避免窗口超出工作区。
const EDGE_ALLOWANCE: f64 = 96.0;

/// 显式声明 Per-Monitor V2 DPI 感知。
///
/// Tauri 生成的清单只声明了 `dpiAware`，在 150% 缩放的机器上会出现
/// 「tao 认为缩放是 1.0、WebView2 按 1.5 渲染」的不一致：页面被放大 1.5 倍
/// 后超出窗口，右侧内容被裁掉。这里在创建任何窗口之前设置进程级 DPI 感知，
/// 让窗口坐标与 WebView2 的缩放因子保持一致。
#[cfg(windows)]
fn enable_per_monitor_dpi_awareness() {
    // DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2 = -4
    const PER_MONITOR_AWARE_V2: isize = -4;

    #[link(name = "user32")]
    extern "system" {
        fn SetProcessDpiAwarenessContext(value: isize) -> i32;
    }

    // 清单里已设置 DPI 感知时调用会失败，此时保持现状即可。
    unsafe {
        SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2);
    }
}

#[cfg(not(windows))]
fn enable_per_monitor_dpi_awareness() {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    enable_per_monitor_dpi_awareness();

    tauri::Builder::default()
        .manage(commands::AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::x_auth_status,
            commands::x_restore_session,
            commands::x_current_user,
            commands::x_login,
            commands::x_logout,
            commands::x_activate_guest,
            commands::x_set_query_id,
            commands::x_query_ids,
            commands::x_graphql,
            commands::x_user_by_screen_name,
            commands::x_home_timeline,
            commands::x_trends,
            commands::x_favorite,
            commands::x_retweet,
            commands::x_bookmark,
            commands::x_create_tweet,
            commands::x_user_timeline,
            commands::x_bookmarks,
            commands::x_search_timeline,
            commands::x_notifications,
            commands::x_tweet_detail,
            commands::x_dm_conversations,
            commands::x_dm_messages,
            commands::x_dm_send,
            commands::x_follow,
        ])
        .setup(|app| {
            // 窗口创建与 WebView2 初始化是异步的；setup 阶段同步改尺寸会被
            // 初始化流程覆盖。这里等窗口稳定后再落一次尺寸与位置。
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(400));
                if let Some(window) = handle.get_webview_window("main") {
                    if let Err(error) = fit_phone_ratio(&window) {
                        eprintln!("调整窗口比例失败：{error}");
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("运行 twitter-miuix 失败");
}

/// 在保持 9:19.5 比例的前提下，把窗口缩放到当前显示器可容纳的最大尺寸。
///
/// `monitor.size()` 是物理像素，换成逻辑尺寸后再与设计基准比较；最终用
/// `LogicalSize` 设置，让 Tauri 按当前 DPI 换算成物理像素。
fn fit_phone_ratio(window: &WebviewWindow) -> tauri::Result<()> {
    let scale = window.scale_factor()?;
    let (available_width, available_height) = match window.current_monitor()? {
        Some(monitor) => {
            let size = monitor.size().to_logical::<f64>(scale);
            (size.width, size.height)
        }
        None => (432.0, BASE_HEIGHT),
    };

    let max_height = (available_height - EDGE_ALLOWANCE).clamp(360.0, BASE_HEIGHT);
    let mut height = max_height;
    let mut width = height * PHONE_ASPECT;

    // 极窄屏幕时以宽度为约束，仍保持同一比例。
    let max_width = (available_width - 40.0).max(240.0);
    if width > max_width {
        width = max_width;
        height = width / PHONE_ASPECT;
    }

    window.set_size(Size::Logical(LogicalSize::new(
        width.round(),
        height.round(),
    )))?;
    window.center()?;
    window.show()?;
    window.set_focus()?;
    Ok(())
}