#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use photocraft_core::{DocInspectInfo, ExportResult, PhotocraftClient, PlaceResult};
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, LogicalPosition, Manager, State, Url, WebviewWindow, WindowEvent,
};

struct AppState {
    client: Arc<PhotocraftClient>,
}

fn make_relay_init_script() -> &'static str {
    r#"(function() {
    var host = window.location.host;
    if (!host) return;
    if (window._tauriRelayHost === host) return;
    window._tauriRelayHost = host;

    var proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    var RELAY_URL = proto + '//' + host + '/comfypanel/ws/ext_bridge?role=embedded';
    var relayWs = null;

    function connectRelay() {
        try {
            relayWs = new WebSocket(RELAY_URL);
            relayWs.onmessage = function(evt) {
                try { window.postMessage(JSON.parse(evt.data), '*'); } catch(e) {}
            };
            relayWs.onclose = function() {
                relayWs = null;
                setTimeout(connectRelay, 3000);
            };
        } catch(e) {
            setTimeout(connectRelay, 5000);
        }
    }
    connectRelay();

    window.uxpHost = {
        postMessage: function(payload) {
            if (relayWs && relayWs.readyState === 1) {
                try { relayWs.send(JSON.stringify(payload)); } catch(e) {}
            }
        }
    };
})();"#
}

#[tauri::command]
async fn bridge_navigate(app: AppHandle, url: String) -> Result<(), String> {
    let parsed_url = Url::parse(&url).map_err(|e| e.to_string())?;

    if let Some(win) = app.get_webview_window("comfy_bridge") {
        win.navigate(parsed_url).map_err(|e| e.to_string())?;
    } else {
        let init_script = make_relay_init_script();
        tauri::WebviewWindowBuilder::new(
            &app,
            "comfy_bridge",
            tauri::WebviewUrl::External(parsed_url),
        )
        .title("ComfyUI - ComfyPanel Companion")
        .inner_size(1280.0, 860.0)
        .center()
        .resizable(true)
        .decorations(true)
        .visible(false)
        .initialization_script(init_script)
        .build()
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn bridge_toggle_view(app: AppHandle, visible: Option<bool>) -> Result<bool, String> {
    if let Some(win) = app.get_webview_window("comfy_bridge") {
        let is_visible = win.is_visible().map_err(|e| e.to_string())?;
        let should_show = visible.unwrap_or(!is_visible);
        if should_show {
            win.show().map_err(|e| e.to_string())?;
            win.set_focus().map_err(|e| e.to_string())?;
        } else {
            win.hide().map_err(|e| e.to_string())?;
        }
        Ok(should_show)
    } else {
        Err("comfy_bridge window not found".to_string())
    }
}

#[tauri::command]
async fn bridge_send(app: AppHandle, payload: serde_json::Value) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("comfy_bridge") {
        let script = format!(
            "window.postMessage({}, '*');",
            serde_json::to_string(&payload).map_err(|e| e.to_string())?
        );
        win.eval(&script).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("comfy_bridge window not found".to_string())
    }
}

fn resolve_data_file_path(app: &AppHandle, rel_path: &str) -> Result<std::path::PathBuf, String> {
    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = std::path::Path::new(rel_path);
    if path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir | std::path::Component::Prefix(_)))
    {
        return Err("Invalid path: parent directory traversal or absolute path is not allowed".to_string());
    }
    Ok(base.join(path))
}

#[tauri::command]
async fn fs_read_data_file(app: AppHandle, rel_path: String) -> Result<String, String> {
    let full = resolve_data_file_path(&app, &rel_path)?;
    std::fs::read_to_string(&full).map_err(|e| format!("read_data_file({rel_path}): {e}"))
}

#[tauri::command]
async fn fs_write_data_file(app: AppHandle, rel_path: String, content: String) -> Result<(), String> {
    let full = resolve_data_file_path(&app, &rel_path)?;
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir({rel_path}): {e}"))?;
    }
    std::fs::write(&full, content).map_err(|e| format!("write_data_file({rel_path}): {e}"))
}

#[tauri::command]
async fn fs_read_binary_file(path: String) -> Result<Vec<u8>, String> {
    let mut clean = path.trim().to_string();
    if let Some(stripped) = clean.strip_prefix("asset://localhost") {
        clean = stripped.to_string();
    } else if let Some(stripped) = clean.strip_prefix("file://") {
        clean = stripped.to_string();
    }
    if let Ok(parsed) = Url::parse(&format!("file://{}", clean)) {
        if let Ok(file_path) = parsed.to_file_path() {
            return std::fs::read(&file_path).map_err(|e| format!("fs_read_binary_file({:?}): {e}", file_path));
        }
    }
    std::fs::read(&clean).map_err(|e| format!("fs_read_binary_file({clean}): {e}"))
}

#[tauri::command]
async fn photocraft_get_status(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.client.is_running().await)
}

#[tauri::command]
async fn photocraft_get_cache_dir(state: State<'_, AppState>) -> Result<String, String> {
    let p = state.client.get_default_cache_dir();
    let _ = std::fs::create_dir_all(&p);
    Ok(p.to_string_lossy().replace('\\', "/"))
}

#[tauri::command]
async fn photocraft_get_doc_info(state: State<'_, AppState>) -> Result<DocInspectInfo, String> {
    state.client.inspect_document().await
}

#[tauri::command]
async fn photocraft_export_document(
    state: State<'_, AppState>,
    target_path: String,
    base_mode: Option<String>,
    crop_mode: Option<String>,
    mask_mode: Option<String>,
    output_mode: Option<String>,
) -> Result<ExportResult, String> {
    let t0 = std::time::Instant::now();
    eprintln!("[Tauri] photocraft_export_document START! output_mode: {:?}, base_mode: {:?}", output_mode, base_mode);
    let res = state.client.export_document(&target_path, base_mode, crop_mode, mask_mode, output_mode).await;
    let elapsed = t0.elapsed();
    match &res {
        Ok(r) => eprintln!("[Tauri] export_document SUCCESS in {:.2?}! has_thumb: {}, thumb_len: {}", elapsed, r.thumbnail_b64.is_some(), r.thumbnail_b64.as_ref().map(|s| s.len()).unwrap_or(0)),
        Err(e) => eprintln!("[Tauri] export_document ERROR in {:.2?}: {}", elapsed, e),
    }
    res
}

#[tauri::command]
async fn check_file_exists(path: String) -> bool {
    let clean_path = path.trim_start_matches("file://");
    std::path::Path::new(clean_path).exists()
}

#[tauri::command]
async fn pick_local_file(
    app: AppHandle,
    extensions: Vec<String>,
    target_folder: Option<String>,
) -> Result<serde_json::Value, String> {
    #[cfg(target_os = "macos")]
    let selected_path = {
        let type_clause = if extensions.is_empty() {
            String::new()
        } else {
            let types_str = extensions
                .iter()
                .map(|ext| format!("\"{}\"", ext))
                .collect::<Vec<_>>()
                .join(", ");
            format!(" of type {{{}}}", types_str)
        };
        let script = format!(
            "POSIX path of (choose file with prompt \"Select File\"{})",
            type_clause
        );
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| format!("Failed to launch file picker: {}", e))?;
        if output.status.success() {
            let res = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if res.is_empty() {
                return Ok(serde_json::json!({ "success": false, "error": "User cancelled" }));
            }
            res
        } else {
            return Ok(serde_json::json!({ "success": false, "error": "User cancelled" }));
        }
    };

    #[cfg(target_os = "windows")]
    let selected_path = {
        let filter = if extensions.is_empty() {
            "All Files (*.*)|*.*".to_string()
        } else {
            let exts = extensions
                .iter()
                .map(|ext| format!("*.{}", ext))
                .collect::<Vec<_>>()
                .join(";");
            format!("Supported Files ({})|{}|All Files (*.*)|*.*", exts, exts)
        };
        let ps_cmd = format!(
            "Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.OpenFileDialog; $f.Filter = '{}'; if ($f.ShowDialog() -eq 'OK') {{ $f.FileName }}",
            filter
        );
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .output()
            .map_err(|e| format!("Failed to launch Windows file picker: {}", e))?;
        if output.status.success() {
            let res = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if res.is_empty() {
                return Ok(serde_json::json!({ "success": false, "error": "User cancelled" }));
            }
            res
        } else {
            return Ok(serde_json::json!({ "success": false, "error": "User cancelled" }));
        }
    };

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let selected_path = return Err("File picker is not supported on this OS".to_string());

    let src_path = std::path::Path::new(&selected_path);
    if !src_path.exists() {
        return Ok(serde_json::json!({ "success": false, "error": "Selected file does not exist" }));
    }

    let file_name = src_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image.png")
        .to_string();

    let final_path = if let Some(dest_folder) = target_folder {
        if !dest_folder.trim().is_empty() {
            let dest_dir = std::path::Path::new(&dest_folder);
            let _ = std::fs::create_dir_all(dest_dir);
            let dest_file = dest_dir.join(&file_name);
            if let Err(e) = std::fs::copy(src_path, &dest_file) {
                eprintln!("[pick_local_file] Failed to copy file to target dir: {}, using original path", e);
                selected_path
            } else {
                dest_file.to_string_lossy().to_string()
            }
        } else {
            selected_path
        }
    } else {
        selected_path
    };

    let scope = app.asset_protocol_scope();
    let _ = scope.allow_file(std::path::Path::new(&final_path));

    Ok(serde_json::json!({
        "success": true,
        "image": final_path,
        "name": file_name,
        "isLocal": true
    }))
}

#[tauri::command]
async fn allow_comfy_directories(
    app: AppHandle,
    input_dir: Option<String>,
    output_dir: Option<String>,
) -> Result<bool, String> {
    let scope = app.asset_protocol_scope();
    if let Some(ref inp) = input_dir {
        if !inp.trim().is_empty() {
            let p = std::path::Path::new(inp);
            let _ = scope.allow_directory(p, true);
        }
    }
    if let Some(ref out) = output_dir {
        if !out.trim().is_empty() {
            let p = std::path::Path::new(out);
            let _ = scope.allow_directory(p, true);
        }
    }
    Ok(true)
}

fn get_safe_temp_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("app_cache_dir: {e}"))?
        .join("temp");
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir temp: {e}"))?;
    Ok(dir)
}

#[tauri::command]
async fn get_temp_files_info(app: AppHandle) -> Result<serde_json::Value, String> {
    let dir = get_safe_temp_dir(&app)?;
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                let native_path = path.to_string_lossy().replace('\\', "/");
                let mtime = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                files.push(serde_json::json!({
                    "name": name,
                    "nativePath": native_path,
                    "mtime": mtime,
                }));
            }
        }
    }
    Ok(serde_json::json!({ "success": true, "files": files }))
}

#[tauri::command]
async fn download_url_to_file(
    app: AppHandle,
    url: String,
    file_name: String,
) -> Result<String, String> {
    let parsed = Url::parse(&url).map_err(|e| format!("Invalid URL: {e}"))?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err("Only HTTP/HTTPS protocols are supported".to_string());
    }
    let safe_name = std::path::Path::new(&file_name)
        .file_name()
        .ok_or_else(|| "Invalid file name".to_string())?;
    let final_path = get_safe_temp_dir(&app)?.join(safe_name);
    let resp = reqwest_like_download(&url).await.map_err(|e| e.to_string())?;
    std::fs::write(&final_path, resp).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(final_path.to_string_lossy().replace('\\', "/"))
}

async fn reqwest_like_download(url: &str) -> Result<Vec<u8>, String> {
    let client = std::process::Command::new("curl")
        .args(["-sSL", url])
        .output()
        .map_err(|e| format!("Failed to execute curl: {}", e))?;
    if client.status.success() {
        Ok(client.stdout)
    } else {
        Err(String::from_utf8_lossy(&client.stderr).to_string())
    }
}

#[tauri::command]
async fn photocraft_create_doc_from_image(
    state: State<'_, AppState>,
    image_path: String,
) -> Result<PlaceResult, String> {
    eprintln!("[Tauri] photocraft_create_doc_from_image called with: {}", image_path);
    state.client.create_doc_from_image(&image_path).await
}

#[tauri::command]
async fn photocraft_place_image(
    state: State<'_, AppState>,
    image_path: String,
    target_x: Option<i32>,
    target_y: Option<i32>,
    mode: Option<String>,
    is_shift: Option<bool>,
    is_auto_insert: Option<bool>,
    layer_bounds: Option<Vec<i32>>,
) -> Result<PlaceResult, String> {
    eprintln!("[Tauri] photocraft_place_image called with: {}, mode: {:?}, is_shift: {:?}, is_auto_insert: {:?}", image_path, mode, is_shift, is_auto_insert);
    state.client.place_image(&image_path, target_x, target_y, mode, is_shift, is_auto_insert, layer_bounds).await
}

#[cfg(target_os = "macos")]
fn is_photocraft_active() -> bool {
    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject};

    unsafe {
        let workspace_cls = match AnyClass::get(c"NSWorkspace") {
            Some(c) => c,
            None => return false,
        };
        let workspace_cls_ptr = workspace_cls as *const AnyClass as *mut AnyObject;
        let workspace: *mut AnyObject = msg_send![workspace_cls_ptr, sharedWorkspace];
        if workspace.is_null() {
            return false;
        }
        let app: *mut AnyObject = msg_send![workspace, frontmostApplication];
        if app.is_null() {
            return false;
        }
        let is_hidden: bool = msg_send![app, isHidden];
        if is_hidden {
            return false;
        }
        let bundle_id: *mut AnyObject = msg_send![app, bundleIdentifier];
        if !bundle_id.is_null() {
            let cstr: *const std::ffi::c_char = msg_send![bundle_id, UTF8String];
            if !cstr.is_null() {
                let s = std::ffi::CStr::from_ptr(cstr).to_str().unwrap_or("");
                if s.to_lowercase().contains("photocraft") || s.contains("comfypanel") {
                    return true;
                }
            }
        }
        let app_name: *mut AnyObject = msg_send![app, localizedName];
        if !app_name.is_null() {
            let cstr: *const std::ffi::c_char = msg_send![app_name, UTF8String];
            if !cstr.is_null() {
                let s = std::ffi::CStr::from_ptr(cstr).to_str().unwrap_or("");
                if s.to_lowercase().contains("photocraft") {
                    return true;
                }
            }
        }
        false
    }
}

#[tauri::command]
async fn open_external_url(url: String) -> Result<(), String> {
    let parsed = Url::parse(&url).map_err(|e| format!("Invalid URL: {e}"))?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err("Only HTTP/HTTPS URLs are allowed".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(parsed.as_str())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(parsed.as_str())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(parsed.as_str())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn is_photocraft_process_running() -> bool {
    let output = std::process::Command::new("pgrep")
        .args(["-f", "PhotoCraft.app/Contents/MacOS/PhotoCraft"])
        .output();
    match output {
        Ok(out) => out.status.success(),
        Err(_) => true,
    }
}

fn position_window_right(win: &WebviewWindow) {
    if let Ok(Some(monitor)) = win.current_monitor() {
        let screen_size = monitor.size();
        let scale_factor = monitor.scale_factor();
        let logical_screen_w = screen_size.width as f64 / scale_factor;
        let logical_screen_h = screen_size.height as f64 / scale_factor;

        let win_size = win.outer_size().unwrap_or(tauri::PhysicalSize::new(320, 960));
        let win_w = win_size.width as f64 / scale_factor;
        let win_h = win_size.height as f64 / scale_factor;

        let target_x = (logical_screen_w - win_w).max(0.0);
        let target_y = ((logical_screen_h - win_h) / 2.0).max(40.0);

        let _ = win.set_position(LogicalPosition::new(target_x, target_y));
    }
}

fn start_lifecycle_tracker(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last_visible = false;
        let mut pc_ever_ran = false;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(300));
            #[cfg(target_os = "macos")]
            {
                let running = is_photocraft_process_running();
                if running {
                    pc_ever_ran = true;
                } else if pc_ever_ran {

                    println!("[ComfyPanel] PhotoCraft process exited. Exiting ComfyPanel...");
                    app.exit(0);
                    break;
                }

                let active = is_photocraft_active();
                if active != last_visible {
                    last_visible = active;
                    if let Some(win) = app.get_webview_window("main") {
                        if active {
                            let _ = win.show();
                            let _ = win.set_always_on_top(true);
                        } else {
                            let _ = win.hide();
                        }
                    }
                }
            }
        }
    });
}

#[cfg(target_os = "macos")]
fn apply_non_activating_macos(win: &tauri::WebviewWindow) {
    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    const NS_NONACTIVATING_PANEL_MASK: usize = 128;
    const NS_FLOATING_WINDOW_LEVEL: isize = 3;

    let handle = match win.window_handle() {
        Ok(h) => h,
        Err(_) => return,
    };
    let ns_view = match handle.as_raw() {
        RawWindowHandle::AppKit(h) => h.ns_view.as_ptr() as *mut AnyObject,
        _ => return,
    };

    unsafe {
        let ns_win: *mut AnyObject = msg_send![ns_view, window];
        if !ns_win.is_null() {
            use std::sync::Once;
            static REGISTER_PANEL_CLASS: Once = Once::new();

            extern "C" fn yes_getter(_this: *mut AnyObject, _cmd: *const std::ffi::c_void) -> bool {
                true
            }

            REGISTER_PANEL_CLASS.call_once(|| {
                extern "C" {
                    fn objc_allocateClassPair(superclass: *const AnyClass, name: *const std::ffi::c_char, extraBytes: usize) -> *mut AnyClass;
                    fn objc_registerClassPair(cls: *mut AnyClass);
                    fn class_addMethod(
                        cls: *mut AnyClass,
                        name: *const std::ffi::c_void,
                        imp: extern "C" fn(*mut AnyObject, *const std::ffi::c_void) -> bool,
                        types: *const std::ffi::c_char,
                    ) -> bool;
                    fn sel_registerName(str: *const std::ffi::c_char) -> *const std::ffi::c_void;
                }

                if let Some(panel_class) = AnyClass::get(c"NSPanel") {
                    let new_cls = objc_allocateClassPair(panel_class as *const AnyClass, c"TurboBarPanel".as_ptr() as *const _, 0);
                    if !new_cls.is_null() {
                        let sel_key = sel_registerName(c"canBecomeKeyWindow".as_ptr() as *const _);
                        let types = c"c@:".as_ptr() as *const _;
                        class_addMethod(new_cls, sel_key, yes_getter, types);
                        objc_registerClassPair(new_cls);
                    }
                }
            });

            if let Some(turbo_panel_class) = AnyClass::get(c"TurboBarPanel") {
                extern "C" {
                    fn object_setClass(obj: *mut AnyObject, cls: *const AnyClass) -> *const AnyClass;
                }
                object_setClass(ns_win, turbo_panel_class);
            }

            let mask: usize = msg_send![ns_win, styleMask];
            let _: () = msg_send![ns_win, setStyleMask: mask | NS_NONACTIVATING_PANEL_MASK];
            let _: () = msg_send![ns_win, setLevel: NS_FLOATING_WINDOW_LEVEL];
            let _: () = msg_send![ns_win, setBecomesKeyOnlyIfNeeded: true];
            let _: () = msg_send![ns_win, setHidesOnDeactivate: false];
            let _: () = msg_send![ns_win, setFloatingPanel: true];
            let _: () = msg_send![ns_win, setAcceptsMouseMovedEvents: true];
            let _: () = msg_send![ns_win, setCollectionBehavior: 2usize | 256usize];
        }
    }
}

#[cfg(target_os = "windows")]
fn apply_non_activating_windows(win: &tauri::WebviewWindow) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE,
    };

    let handle = match win.window_handle() {
        Ok(h) => h,
        Err(_) => return,
    };
    let hwnd = match handle.as_raw() {
        RawWindowHandle::Win32(h) => h.hwnd.get() as isize,
        _ => return,
    };

    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | WS_EX_NOACTIVATE as isize);
    }
}

fn main() {
    let client = Arc::new(PhotocraftClient::new());

    client.ensure_env_configured();

    tauri::Builder::default()
        .manage(AppState { client })
        .invoke_handler(tauri::generate_handler![
            photocraft_get_status,
            photocraft_get_cache_dir,
            photocraft_get_doc_info,
            photocraft_export_document,
            photocraft_create_doc_from_image,
            photocraft_place_image,
            check_file_exists,
            download_url_to_file,
            bridge_navigate,
            bridge_toggle_view,
            bridge_send,
            open_external_url,
            fs_read_data_file,
            fs_write_data_file,
            fs_read_binary_file,
            get_temp_files_info,
            pick_local_file,
            allow_comfy_directories
        ])
        .setup(|app| {

            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                if let Ok(cache_dir) = app.path().app_cache_dir() {
                    let net_cache = cache_dir.join("WebKit").join("NetworkCache");
                    if net_cache.exists() {
                        let _ = std::fs::remove_dir_all(&net_cache);
                    }
                }
            }
            #[cfg(target_os = "windows")]
            if let Ok(local_data_dir) = app.path().app_local_data_dir() {
                let web_cache = local_data_dir
                    .join("EBWebView")
                    .join("Default")
                    .join("Cache");
                if web_cache.exists() {
                    let _ = std::fs::remove_dir_all(&web_cache);
                }
            }

            if let Some(win) = app.get_webview_window("main") {
                position_window_right(&win);
            }

            if let Some(win) = app.get_webview_window("turbo") {
                #[cfg(target_os = "macos")]
                apply_non_activating_macos(&win);

                #[cfg(target_os = "windows")]
                apply_non_activating_windows(&win);
            }

            let handle = app.handle().clone();

            #[cfg(target_os = "macos")]
            if let Ok(home) = std::env::var("HOME") {
                let comfy_root = std::path::Path::new(&home).join("ComfyUI");
                let scope = app.asset_protocol_scope();
                let _ = scope.allow_directory(comfy_root.join("output"), true);
                let _ = scope.allow_directory(comfy_root.join("input"), true);
                let _ = scope.allow_directory(comfy_root.join("temp"), true);
            }
            #[cfg(target_os = "windows")]
            if let Ok(profile) = std::env::var("USERPROFILE") {
                let comfy_root = std::path::Path::new(&profile).join("ComfyUI");
                let scope = app.asset_protocol_scope();
                let _ = scope.allow_directory(comfy_root.join("output"), true);
                let _ = scope.allow_directory(comfy_root.join("input"), true);
                let _ = scope.allow_directory(comfy_root.join("temp"), true);
            }

            start_lifecycle_tracker(handle);

            let toggle = MenuItem::with_id(app, "toggle", "Show / Hide ComfyPanel", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit ComfyPanel", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle, &quit])?;

            let tray_img = image::load_from_memory(include_bytes!("../icons/tray-icon.png"))
                .expect("failed to load tray icon")
                .into_rgba8();
            let (w, h) = tray_img.dimensions();
            let tray_icon = Image::new_owned(tray_img.into_raw(), w, h);
            TrayIconBuilder::new()
                .icon(tray_icon)
                .icon_as_template(true)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "toggle" => {
                            if let Some(win) = app.get_webview_window("main") {
                                if let Ok(visible) = win.is_visible() {
                                    if visible {
                                        let _ = win.hide();
                                    } else {
                                        let _ = win.show();
                                        let _ = win.set_always_on_top(true);
                                    }
                                }
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {

            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running ComfyPanel");
}