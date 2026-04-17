// 预设管理命令

use super::async_fs;
use super::get_presets_dir;

/// 列出所有预设文件名（异步）
#[tauri::command]
pub async fn list_presets() -> Result<Vec<String>, String> {
    let path = get_presets_dir()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut entries = async_fs::read_dir(&path)
        .await
        .map_err(|e| format!("读取预设列表失败: {}", e))?;
    
    let mut names = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("读取预设条目失败: {}", e))?
    {
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.replace(".json", ""));
        }
    }
    Ok(names)
}

/// 读取指定预设文件内容（异步）
#[tauri::command]
pub async fn read_preset(name: String) -> Result<String, String> {
    let path = get_presets_dir()?.join(format!("{}.json", name));
    async_fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取预设失败: {}", e))
}

/// 保存预设到文件（异步）
#[tauri::command]
pub async fn save_preset(name: String, content: String) -> Result<(), String> {
    let presets_dir = get_presets_dir()?;
    // 确保预设目录存在
    async_fs::create_dir_all(&presets_dir)
        .await
        .map_err(|e| format!("创建预设目录失败: {}", e))?;
    let path = presets_dir.join(format!("{}.json", name));
    async_fs::write(&path, content)
        .await
        .map_err(|e| format!("保存预设失败: {}", e))
}

/// 删除指定预设文件（异步）
#[tauri::command]
pub async fn delete_preset(name: String) -> Result<(), String> {
    let path = get_presets_dir()?.join(format!("{}.json", name));
    async_fs::remove_file(&path)
        .await
        .map_err(|e| format!("删除预设失败: {}", e))
}

/// 读取所有预设（合并命令，避免 N+1 问题）
/// 一次返回所有预设的完整数据
#[tauri::command]
pub async fn read_all_presets() -> Result<String, String> {
    let presets_dir = get_presets_dir()?;
    if !presets_dir.exists() {
        return Ok("[]".to_string());
    }
    
    let mut entries = async_fs::read_dir(&presets_dir)
        .await
        .map_err(|e| format!("读取预设目录失败: {}", e))?;
    
    let mut presets: Vec<serde_json::Value> = Vec::new();
    
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("读取预设条目失败: {}", e))?
    {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        
        if name.ends_with(".json") {
            let preset_name = name.trim_end_matches(".json");
            let path = entry.path();
            
            if let Ok(content) = async_fs::read_to_string(&path).await {
                if let Ok(mut data) = serde_json::from_str::<serde_json::Value>(&content) {
                    // 添加预设名称
                    if let Some(obj) = data.as_object_mut() {
                        obj.insert("name".to_string(), serde_json::json!(preset_name));
                    }
                    presets.push(data);
                }
            }
        }
    }
    
    // 按更新时间倒序排列
    presets.sort_by(|a, b| {
        let a_time = a.get("updatedAt").and_then(|t| t.as_str()).unwrap_or("");
        let b_time = b.get("updatedAt").and_then(|t| t.as_str()).unwrap_or("");
        b_time.cmp(a_time)
    });
    
    Ok(serde_json::to_string(&presets).unwrap_or("[]".to_string()))
}
