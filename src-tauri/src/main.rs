// OMOSwitcher 应用程序入口点
// 防止额外的控制台窗口在 Windows 上出现

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    omo_switcher::run()
}
