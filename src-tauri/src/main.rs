// Windows 发布版为 GUI 子系统；调试版保留控制台便于查看日志。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    twitter_miuix_lib::run()
}