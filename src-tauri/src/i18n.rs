//! Native text (tray menu, window titles). The pages resolve "follow system" and report
//! the language with `set_ui_lang`; everything else is translated in the pages.

/// `lang`: "zh-TW", "zh-CN" or "en".
pub fn tr(lang: &str, key: &'static str) -> &'static str {
    let i = match lang {
        "en" => 0,
        "zh-CN" => 1,
        _ => return key,
    };
    let row: [&str; 2] = match key {
        "設定…" => ["Settings…", "设置…"],
        "立即更新用量" => ["Refresh usage now", "立即更新用量"],
        "重新定位膠囊" => ["Reposition capsule", "重新定位胶囊"],
        "結束 DW Bar" => ["Quit DW Bar", "退出 DW Bar"],
        "DW Bar 設定" => ["DW Bar Settings", "DW Bar 设置"],
        "登入" => ["Sign in to", "登录"],
        _ => return key,
    };
    row[i]
}

/// The language to start with, before a page has reported one.
pub fn initial(setting: &str) -> String {
    match setting {
        "zh-TW" | "zh-CN" | "en" => setting.to_string(),
        _ => "zh-TW".to_string(),
    }
}
