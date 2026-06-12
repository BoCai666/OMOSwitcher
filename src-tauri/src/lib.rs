// OMOSwitcher Tauri 应用库
// 包含应用初始化和命令定义

mod commands;
mod monitor;
mod quota;
mod sync;
mod bubble;

use std::fmt;
use tauri::Manager;
use tauri::menu::{MenuBuilder, MenuItem};
use tauri::tray::TrayIconBuilder;
use tracing_subscriber::fmt::{FormatEvent, FormatFields, FmtContext};
use tracing_subscriber::registry::LookupSpan;

/// 自定义日志格式：本地时间 + 精确到秒 + 去掉 crate 名前缀
struct CompactFormat;

/// 编译时解析时间格式模板
const TIME_FORMAT: &[time::format_description::FormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

impl<S, N> FormatEvent<S, N> for CompactFormat
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> fmt::Result {
        // 本地时间，精确到秒（灰色）
        let now = time::OffsetDateTime::now_local()
            .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
        if writer.has_ansi_escapes() {
            write!(writer, "\x1b[2m{}\x1b[0m ", now.format(TIME_FORMAT).unwrap_or_default())?;
        } else {
            write!(writer, "{} ", now.format(TIME_FORMAT).unwrap_or_default())?;
        }

        // 日志级别（右对齐 5 字符，保留原始颜色）
        let level = event.metadata().level();
        if writer.has_ansi_escapes() {
            // 与 tracing-subscriber 默认配色一致
            let color = match *level {
                tracing::Level::TRACE => "\x1b[35m",
                tracing::Level::DEBUG => "\x1b[36m",
                tracing::Level::INFO => "\x1b[32m",
                tracing::Level::WARN => "\x1b[33m",
                tracing::Level::ERROR => "\x1b[31m",
            };
            write!(writer, "{}{:>5}\x1b[0m ", color, level)?;
        } else {
            write!(writer, "{:>5} ", level)?;
        }

        // 模块路径（灰色，去掉 omoswitcher:: 前缀）
        let target = event
            .metadata()
            .target()
            .strip_prefix("omoswitcher::")
            .unwrap_or(event.metadata().target());
        if writer.has_ansi_escapes() {
            write!(writer, "\x1b[2m{}\x1b[0m: ", target)?;
        } else {
            write!(writer, "{}: ", target)?;
        }

        // 日志消息
        ctx.format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

/// 初始化 tracing 日志系统
/// - Debug 模式 (cargo tauri dev): INFO 级别，显示所有追踪日志
/// - Release 模式 (cargo tauri build): WARN 级别，仅显示警告和错误
/// - 环境变量 RUST_LOG 可覆盖默认级别（如 RUST_LOG=debug）
fn init_logging() {
    use tracing_subscriber::EnvFilter;

    // 根据编译模式设置默认日志级别
    #[cfg(debug_assertions)]
    let default_level = "info";
    #[cfg(not(debug_assertions))]
    let default_level = "warn";

    // 优先使用 RUST_LOG 环境变量，否则使用编译模式对应的默认级别
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .event_format(CompactFormat)
        .init();

    tracing::info!("[App] 日志系统初始化完成，模式={}", default_level);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志系统（必须在所有 tracing 调用之前）
    init_logging();

    // 初始化 Monitor 状态（存储、配置、证书）
    let monitor_state = monitor::command::MonitorCommandState::new()
        .expect("Monitor 初始化失败");

    // 初始化 Sync 状态
    let sync_state = sync::command::SyncCommandState::default();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_keyring::init())
        .plugin(tauri_plugin_process::init())
        // 创建系统托盘
        .setup(|app| {
            let show_item = MenuItem::with_id(app, "show", "显示 OMOSwitcher", true, None::<&str>)?;
            let bubble_text = if app.get_webview_window("bubble").is_some() {
                "关闭悬浮球"
            } else {
                "开启悬浮球"
            };
            let bubble_item = MenuItem::with_id(app, "toggle_bubble", bubble_text, true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&bubble_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("OMOSwitcher")
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "toggle_bubble" => {
                            let is_visible = app.get_webview_window("bubble").is_some();
                            if is_visible {
                                if let Some(window) = app.get_webview_window("bubble") {
                                    let _ = window.close();
                                }
                                let _ = bubble_item.set_text("开启悬浮球");
                            } else {
                                use tauri::WebviewUrl;
                                use tauri::WebviewWindowBuilder;
                                let _ = WebviewWindowBuilder::new(app, "bubble", WebviewUrl::App("bubble.html".into()))
                                    .title("OMOSwitcher - 悬浮球")
                                    .inner_size(80.0, 80.0)
                                    .always_on_top(true)
                                    .skip_taskbar(true)
                                    .decorations(false)
                                    .transparent(true)
                                    .shadow(false)
                                    .resizable(false)
                                    .visible(true)
                                    .build();
                                let _ = bubble_item.set_text("关闭悬浮球");
                            }
                        }
                        "quit" => {
                            commands::launch::cleanup_opencode_child();
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键点击直接显示窗口
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;

            // 启动时根据保存的 enabled 状态自动恢复悬浮球
            let bubble_should_restore = bubble::commands::read_bubble_enabled(&app.handle());
            if bubble_should_restore && app.get_webview_window("bubble").is_none() {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = bubble::commands::create_bubble(app_handle).await {
                        tracing::error!("[Bubble] 启动时恢复悬浮球失败: {}", e);
                    }
                });
            }

            Ok(())
        })
        // 注册 Monitor 状态管理
        .manage(monitor_state)
        // 注册 Sync 状态管理
        .manage(sync_state)
        .invoke_handler(tauri::generate_handler![
            // 配置管理命令
            commands::read_config,
            commands::write_config,
            commands::read_opencode_config,
            // 预设管理命令
            commands::list_presets,
            commands::read_preset,
            commands::save_preset,
            commands::delete_preset,
            commands::read_all_presets,
            // 模型管理命令
            commands::read_models,
            commands::write_models,
            commands::read_models_with_fallback,
            // 设置管理命令
            commands::read_settings,
            commands::write_settings,
            // 启动命令
            commands::launch_opencode,
            // 浏览器命令
            commands::open_url_in_browser,
            // 热重载命令
            commands::detect_opencode_server,
            commands::hot_reload_config,
            commands::dispose_instance,
            commands::get_active_sessions,
            commands::resume_session,
            commands::dispose_and_resume,
            // 端口管理命令
            commands::kill_port_process,
            // Monitor 代理服务命令
            commands::start_monitor_service,
            commands::stop_monitor_service,
            commands::get_monitor_status,
            // 证书相关命令
            commands::check_ca_cert_exists,
            // Monitor 端口配置命令
            commands::get_monitor_ports_config,
            // 模型注册表命令
            commands::read_opencode_models_cache,
            commands::get_available_provider_ids,
            commands::get_custom_provider_ids,
            commands::add_custom_provider,
            commands::delete_custom_provider,
            // 额度查询命令
            quota::fetch_all_provider_quotas,
            quota::zhipu::fetch_zhipu_usage_details,
            // ========== Monitor 数据查询命令 ==========
            monitor::command::monitor_get_requests,
            monitor::command::monitor_get_request,
            monitor::command::monitor_get_response,
            monitor::command::monitor_get_metrics,
            monitor::command::monitor_get_mcp_calls,
            monitor::command::monitor_get_stats_summary,
            monitor::command::monitor_get_daily_records,
            monitor::command::monitor_get_domain_stats,
            monitor::command::monitor_get_all_models,
            monitor::command::monitor_get_delta,
            // ========== Monitor 数据操作命令 ==========
            monitor::command::monitor_clear_data,
            monitor::command::monitor_export_data,
            monitor::command::monitor_backup,
            monitor::command::monitor_get_backups,
            // ========== Monitor 配置操作命令 ==========
            monitor::command::monitor_get_config,
            monitor::command::monitor_update_config,
            monitor::command::monitor_get_domains,
            monitor::command::monitor_add_domain,
            monitor::command::monitor_remove_domain,
            monitor::command::monitor_get_pricing,
            monitor::command::monitor_update_pricing,
            // ========== Monitor 证书操作命令 ==========
            monitor::command::monitor_cert_status,
            monitor::command::monitor_health,
            // ========== Sync 同步命令 ==========
            sync::command::sync_get_auth_state,
            sync::command::sync_start_device_login,
            sync::command::sync_complete_device_login,
            sync::command::sync_login_with_pat,
            sync::command::sync_logout,
            sync::command::sync_get_status,
            sync::command::sync_upload,
            sync::command::sync_download,
            sync::command::sync_perform,
            sync::command::sync_resolve_conflict,
            sync::command::sync_cancel_device_login,
            sync::command::sync_start_oauth_login,
            sync::command::sync_cancel_oauth_login,
            // ========== Bubble 悬浮球命令 ==========
            bubble::commands::create_bubble,
            bubble::commands::destroy_bubble,
            bubble::commands::toggle_bubble,
            bubble::commands::get_bubble_settings,
            bubble::commands::save_bubble_position,
            bubble::commands::is_bubble_visible,
        ])
        .on_window_event(|window, event| {
            // 窗口关闭时的处理
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                tracing::info!("[App] 窗口关闭... label={}", window.label());
                if window.label() == "main" {
                    // 清理代理模式启动的 opencode 子进程
                    commands::launch::cleanup_opencode_child();
                    // 真正退出应用：主窗口关闭时一并退出整个进程（包括悬浮球）
                    window.app_handle().exit(0);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用时出错");
}
