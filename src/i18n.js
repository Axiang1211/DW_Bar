// Languages: 繁體中文 (source text), 简体中文, English.
// Keys are the Traditional Chinese strings used in the code; {name} placeholders are filled in.
// `t()` translates UI text, `tb()` translates text that comes from the background engine
// (window labels, notices), `localizePage()` translates the static HTML of a page.

export const LANGS = [
  ["auto", "跟隨系統 · System"],
  ["zh-TW", "繁體中文"],
  ["zh-CN", "简体中文"],
  ["en", "English"],
];

// [English, 简体中文]
const D = {
  // ── navigation & pages ──
  "DW Bar 設定": ["DW Bar Settings", "DW Bar 设置"],
  "設定分類": ["Settings sections", "设置分类"],
  "用量": ["Usage", "用量"],
  "位置": ["Position", "位置"],
  "外觀": ["Appearance", "外观"],
  "AI 帳號": ["AI accounts", "AI 账号"],
  "一般": ["General", "常规"],
  "診斷": ["Diagnostics", "诊断"],
  "立即更新": ["Refresh now", "立即更新"],
  "更新中…": ["Refreshing…", "更新中…"],
  "設定": ["Settings", "设置"],
  "關閉": ["Close", "关闭"],
  "AI 用量": ["AI usage", "AI 用量"],
  "AI 用量面板": ["AI usage panel", "AI 用量面板"],
  "管理 AI 帳號": ["Manage AI accounts", "管理 AI 账号"],

  // ── position ──
  "九宮格八向定位：中間是工作列，其他位置會變成懸浮膠囊。": ["Pick a spot on the grid: the centre is the taskbar, every other cell is a floating capsule.", "九宫格八向定位：中间是任务栏，其他位置会变成悬浮胶囊。"],
  "九宮格八向定位：中間是 Dock 上方，其他位置會變成懸浮膠囊。": ["Pick a spot on the grid: the centre sits above the Dock, every other cell is a floating capsule.", "九宫格八向定位：中间是 Dock 上方，其他位置会变成悬浮胶囊。"],
  "膠囊位置": ["Capsule position", "胶囊位置"],
  "工作列位置規則": ["Taskbar placement", "任务栏位置规则"],
  "小工具位置（建議）": ["Widgets spot (recommended)", "小组件位置（推荐）"],
  "固定最左": ["Always far left", "固定最左"],
  "固定系統匣左邊": ["Left of the system tray", "固定系统托盘左边"],
  "水平微調": ["Fine-tune horizontally", "水平微调"],
  "單位為像素，正數往右。可用來避開小工具或其他圖示。": ["In pixels; positive moves right. Use it to clear the widgets button or other icons.", "单位为像素，正数往右。可用来避开小组件或其他图标。"],
  "往左": ["Left", "往左"],
  "往右": ["Right", "往右"],
  "附著方式": ["Attach mode", "附着方式"],
  "嵌入：成為工作列的一部分，跟著工作列移動與隱藏。若顯示異常，改用浮貼。": ["Embedded: becomes part of the taskbar and moves and hides with it. If it looks wrong, switch to floating.", "嵌入：成为任务栏的一部分，跟着任务栏移动与隐藏。若显示异常，改用浮动。"],
  "嵌入工作列": ["Embedded in the taskbar", "嵌入任务栏"],
  "浮貼在上方": ["Float on top", "浮动在上方"],
  "Windows 小工具（天氣）": ["Windows widgets (weather)", "Windows 小组件（天气）"],
  "關掉工作列左邊的天氣小工具，膠囊會移到它原本的位置。": ["Turn off the weather widget on the left of the taskbar; the capsule moves into its place.", "关掉任务栏左边的天气小组件，胶囊会移到它原本的位置。"],
  "全螢幕時自動隱藏": ["Hide in fullscreen", "全屏时自动隐藏"],
  "玩遊戲、看影片、簡報時不顯示。": ["Stays out of the way of games, videos and presentations.", "玩游戏、看视频、演示时不显示。"],
  "工作列 · 取代天氣": ["Taskbar · replaces the weather", "任务栏 · 取代天气"],
  "嵌在工作列裡，跟著工作列移動和自動隱藏；放在哪一側由下方「工作列位置規則」決定。": ["Lives inside the taskbar and moves and auto-hides with it; which side is set by “Taskbar placement” below.", "嵌在任务栏里，跟着任务栏移动和自动隐藏；放在哪一侧由下方“任务栏位置规则”决定。"],
  "左上 · 懸浮 HUD": ["Top left · floating HUD", "左上 · 悬浮 HUD"],
  "右上 · 懸浮 HUD": ["Top right · floating HUD", "右上 · 悬浮 HUD"],
  "左下 · 懸浮 HUD": ["Bottom left · floating HUD", "左下 · 悬浮 HUD"],
  "右下 · 懸浮 HUD": ["Bottom right · floating HUD", "右下 · 悬浮 HUD"],
  "正上方 · 懸浮 HUD": ["Top · floating HUD", "正上方 · 悬浮 HUD"],
  "正下方 · 懸浮 HUD": ["Bottom · floating HUD", "正下方 · 悬浮 HUD"],
  "左側 · 懸浮 HUD": ["Left · floating HUD", "左侧 · 悬浮 HUD"],
  "右側 · 懸浮 HUD": ["Right · floating HUD", "右侧 · 悬浮 HUD"],
  "平常只顯示最需要注意的一個 AI，點一下展開全部，2 分鐘沒操作就收回。": ["Shows only the AI that needs attention; click to expand all, it folds back after 2 minutes.", "平常只显示最需要注意的一个 AI，点一下展开全部，2 分钟没操作就收回。"],
  "橫向排列在螢幕上緣中央。": ["A row centred on the top edge of the screen.", "横向排列在屏幕上缘中央。"],
  "直向排列在螢幕左緣中央。": ["A column centred on the left edge of the screen.", "纵向排列在屏幕左缘中央。"],
  "直向排列在螢幕右緣中央。": ["A column centred on the right edge of the screen.", "纵向排列在屏幕右缘中央。"],
  "橫向排列在工作列正上方中央。": ["A row centred just above the taskbar.", "横向排列在任务栏正上方中央。"],
  "圖示置中時放在工作列最左邊（原生天氣的位置）；圖示靠左時自動移到系統匣左邊，不會蓋到開始按鈕。": ["With centred icons it sits at the far left (where the weather was); with left-aligned icons it moves next to the system tray so it never covers Start.", "图标居中时放在任务栏最左边（原生天气的位置）；图标靠左时自动移到系统托盘左边，不会盖到开始按钮。"],
  "一律放在工作列最左邊。": ["Always at the far left of the taskbar.", "一律放在任务栏最左边。"],
  "一律放在系統匣（時鐘）左邊。": ["Always left of the system tray (clock).", "一律放在系统托盘（时钟）左边。"],
  "Dock 上方 · 懸浮膠囊": ["Above the Dock · floating capsule", "Dock 上方 · 悬浮胶囊"],
  "懸浮在 Dock 正上方中央，跟著你切換的每個桌面空間。": ["Floats centred above the Dock and follows you to every Space.", "悬浮在 Dock 正上方中央，跟着你切换的每个桌面空间。"],

  // ── appearance ──
  "主題": ["Theme", "主题"],
  "跟隨系統會依 Windows 的深色／淺色模式切換。": ["“System” follows your dark / light mode.", "跟随系统会依 Windows 的深色／浅色模式切换。"],
  "跟隨系統": ["System", "跟随系统"],
  "深色": ["Dark", "深色"],
  "淺色": ["Light", "浅色"],
  "背景材質": ["Background material", "背景材质"],
  "用量面板、滑鼠移上去的小卡和膠囊的背景。": ["The background of the usage panel, the hover card and the capsule.", "用量面板、鼠标移上去的小卡和胶囊的背景。"],
  "預設": ["Default", "默认"],
  "毛玻璃": ["Frosted glass", "毛玻璃"],
  "液態玻璃": ["Liquid glass", "液态玻璃"],
  "純色": ["Solid colour", "纯色"],
  "漸層": ["Gradient", "渐变"],
  "液態金屬": ["Liquid metal", "液态金属"],
  "極光": ["Aurora", "极光"],
  "夕陽": ["Sunset", "夕阳"],
  "深海": ["Ocean", "深海"],
  "櫻花": ["Sakura", "樱花"],
  "森林": ["Forest", "森林"],
  "夢翔": ["Dream", "梦翔"],
  "夜空": ["Night", "夜空"],
  "糖果": ["Candy", "糖果"],
  "熔岩": ["Lava", "熔岩"],
  "自訂": ["Custom", "自定义"],
  "液態銀": ["Silver", "液态银"],
  "液態金": ["Gold", "液态金"],
  "玫瑰金": ["Rose gold", "玫瑰金"],
  "鈦黑": ["Titanium", "钛黑"],
  "幻彩": ["Holographic", "幻彩"],
  "原本的深色／淺色面板，跟著上面的主題切換。": ["The standard dark / light panel; follows the theme above.", "原本的深色／浅色面板，跟着上面的主题切换。"],
  "毛玻璃：後方的桌面會變模糊、透出原本的顏色，文字顏色跟著主題。": ["Frosted glass: the desktop behind is blurred but keeps its colours; text follows the theme.", "毛玻璃：后方的桌面会变模糊、透出原本的颜色，文字颜色跟着主题。"],
  "液態玻璃：完全透明、不模糊，只有邊緣的高光和折射色；文字放在淡淡的玻璃卡片上。": ["Liquid glass: fully clear with no blur, just bright edges and refraction; text sits on faint glass cards.", "液态玻璃：完全透明、不模糊，只有边缘的高光和折射色；文字放在淡淡的玻璃卡片上。"],
  "純色：選一個顏色和不透明度，文字會自動選擇深色或淺色。": ["Solid colour: pick a colour and opacity; text switches between dark and light automatically.", "纯色：选一个颜色和不透明度，文字会自动选择深色或浅色。"],
  "漸層：選一組配色，或自訂兩個顏色和角度。": ["Gradient: pick a palette, or choose two colours and an angle.", "渐变：选一组配色，或自定义两个颜色和角度。"],
  "液態金屬：拋光金屬的光澤，面板打開時高光會緩慢流動。": ["Liquid metal: polished metal; the highlight drifts slowly while the panel is open.", "液态金属：抛光金属的光泽，面板打开时高光会缓慢流动。"],
  "玻璃濃度": ["Glass strength", "玻璃浓度"],
  "越低越透明。太透明時文字可能不好讀，可以稍微調高。": ["Lower is clearer. If text gets hard to read, turn it up a little.", "越低越透明。太透明时文字可能不好读，可以稍微调高。"],
  "顏色": ["Colour", "颜色"],
  "顏色 1": ["Colour 1", "颜色 1"],
  "顏色 2": ["Colour 2", "颜色 2"],
  "角度": ["Angle", "角度"],
  "不透明度": ["Opacity", "不透明度"],
  "低於 95% 時，後方的桌面會變模糊（毛玻璃效果）。": ["Below 95% the desktop behind is blurred (frosted effect).", "低于 95% 时，后方的桌面会变模糊（毛玻璃效果）。"],
  "圖示風格": ["Icon style", "图标风格"],
  "圈圈中間顯示各家的官方圖示（從官網載入），可選擇不同風格。": ["Each ring shows the service's own icon (loaded from its website) in the style you choose.", "圈圈中间显示各家的官方图标（从官网加载），可选择不同风格。"],
  "極簡線條": ["Minimal", "极简线条"],
  "賽博霓虹": ["Neon", "赛博霓虹"],
  "像素風": ["Pixel", "像素风"],
  "原色": ["Original", "原色"],
  "半透明視窗": ["Translucent windows", "半透明窗口"],
  "讓檔案總管和其他程式視窗的標題列變成半透明（亞克力）。不會修改其他程式，關閉或結束 DW Bar 時還原。": ["Gives File Explorer and other windows a translucent (acrylic) title bar. Other apps aren't modified; everything is restored when you turn it off or quit DW Bar.", "让文件资源管理器和其他程序窗口的标题栏变成半透明（亚克力）。不会修改其他程序，关闭或退出 DW Bar 时还原。"],
  "顯示百分比": ["Show percentage", "显示百分比"],
  "在圖示旁顯示 5 小時額度剩餘 %。關閉時只顯示圖示，滑鼠停留可看詳細。": ["Show the 5-hour % left next to each icon. When off, hover an icon for details.", "在图标旁显示 5 小时额度剩余 %。关闭时只显示图标，鼠标停留可看详细。"],
  "外圈上半弧：5 小時額度剩餘": ["Outer arc: 5-hour limit left", "外圈上半弧：5 小时额度剩余"],
  "內圈：每週額度剩餘": ["Inner ring: weekly limit left", "内圈：每周额度剩余"],
  "綠 ≥ 50%": ["Green ≥ 50%", "绿 ≥ 50%"],
  "黃 ≥ 20%": ["Yellow ≥ 20%", "黄 ≥ 20%"],
  "紅 < 20%": ["Red < 20%", "红 < 20%"],

  // ── accounts ──
  "連結帳號後就能顯示用量。登入在官方網站上完成，DW Bar 不會看到你的密碼；登入狀態只存在這台電腦。": ["Link an account to see its usage. You sign in on the service's own website, so DW Bar never sees your password; the sign-in stays on this computer.", "关联账号后就能显示用量。登录在官方网站上完成，DW Bar 不会看到你的密码；登录状态只保存在这台电脑。"],
  "清除登入資料": ["Clear sign-in data", "清除登录数据"],
  "登出 DW Bar 裡的所有帳號，並刪除這台電腦上 DW Bar 保存的登入狀態。": ["Signs out every account in DW Bar and deletes the sign-in data DW Bar keeps on this computer.", "退出 DW Bar 里的所有账号，并删除这台电脑上 DW Bar 保存的登录状态。"],
  "清除": ["Clear", "清除"],
  "Claude 帳號": ["Claude account", "Claude 账号"],
  "ChatGPT 帳號": ["ChatGPT account", "ChatGPT 账号"],
  "Google 帳號": ["Google account", "Google 账号"],
  "Cursor 帳號": ["Cursor account", "Cursor 账号"],
  "Windsurf 帳號": ["Windsurf account", "Windsurf 账号"],
  "DeepSeek 帳號": ["DeepSeek account", "DeepSeek 账号"],
  "帳號": ["account", "账号"],
  "連結 Gemini（Google 帳號）": ["Link Gemini (Google account)", "关联 Gemini（Google 账号）"],
  "連結 {account}": ["Link {account}", "关联 {account}"],
  "chatgpt.com，讀取 Codex 額度": ["chatgpt.com, reads your Codex limits", "chatgpt.com，读取 Codex 额度"],
  "cursor.com，讀取本期方案額度": ["cursor.com, reads this billing period's usage", "cursor.com，读取本期套餐额度"],
  "windsurf.com，讀取每日／每週額度": ["windsurf.com, reads daily / weekly limits", "windsurf.com，读取每日／每周额度"],
  "platform.deepseek.com，讀取 API 餘額（DSH）": ["platform.deepseek.com, reads your API balance (DSH)", "platform.deepseek.com，读取 API 余额（DSH）"],
  "打開 Antigravity 時自動讀取，不需要登入": ["Read automatically while Antigravity is open, no sign-in needed", "打开 Antigravity 时自动读取，不需要登录"],
  "已連結 {account}": ["Linked {account}", "已关联 {account}"],
  "尚未連結 {account}（{site}）": ["{account} not linked ({site})", "尚未关联 {account}（{site}）"],
  "正在讀取 Antigravity": ["Reading Antigravity", "正在读取 Antigravity"],
  "中斷連結": ["Unlink", "取消关联"],
  "這台電腦上有：{list}": ["Found on this computer: {list}", "这台电脑上有：{list}"],
  "桌面 App": ["desktop app", "桌面应用"],
  "、": [", ", "、"],
  "命令列工具": ["command-line tool", "命令行工具"],
  "這台電腦上沒有偵測到它的 App，連結帳號就能顯示用量。": ["Its app isn't on this computer; link the account to see usage.", "这台电脑上没有检测到它的应用，关联账号就能显示用量。"],
  "進階：指定命令列工具路徑": ["Advanced: command-line tool path", "高级：指定命令行工具路径"],
  "選填，例如 C:\\Users\\你\\.local\\bin\\{exe}.exe": ["Optional, e.g. C:\\Users\\you\\.local\\bin\\{exe}.exe", "选填，例如 C:\\Users\\你\\.local\\bin\\{exe}.exe"],
  "在工作列顯示": ["Show in the taskbar", "在任务栏显示"],
  "請在跳出的視窗中登入": ["Sign in in the window that opened", "请在弹出的窗口中登录"],
  "正在重新偵測…": ["Detecting again…", "正在重新检测…"],
  "已清除登入資料": ["Sign-in data cleared", "已清除登录数据"],
  "已儲存": ["Saved", "已保存"],

  // ── general ──
  "語言 · Language": ["Language · 語言", "语言 · Language"],
  "跟隨系統會依電腦的顯示語言選擇。": ["“System” follows your computer's display language.", "跟随系统会依电脑的显示语言选择。"],
  "開機時啟動": ["Start at login", "开机时启动"],
  "登入電腦後自動開啟 DW Bar。": ["Open DW Bar automatically when you sign in.", "登录电脑后自动打开 DW Bar。"],
  "Windows 在「工作管理員 → 啟動應用程式」把 DW Bar 停用了，所以開機不會啟動。把這個開關關掉再打開，就會重新啟用。": ["Windows has DW Bar turned off in Task Manager → Startup apps, so it won't start at login. Turn this switch off and on again to re-enable it.", "Windows 在“任务管理器 → 启动应用”中把 DW Bar 禁用了，所以开机不会启动。把这个开关关掉再打开，就会重新启用。"],
  "更新頻率": ["Refresh interval", "更新频率"],
  "每次更新會在背景短暫執行一次各家的命令列工具。": ["Each refresh briefly checks every linked service in the background.", "每次更新会在后台短暂查询一次各家的用量。"],
  "每 2 分鐘": ["Every 2 minutes", "每 2 分钟"],
  "每 5 分鐘": ["Every 5 minutes", "每 5 分钟"],
  "每 10 分鐘": ["Every 10 minutes", "每 10 分钟"],
  "每 15 分鐘": ["Every 15 minutes", "每 15 分钟"],
  "每 30 分鐘": ["Every 30 minutes", "每 30 分钟"],
  "每小時": ["Every hour", "每小时"],
  "結束 DW Bar": ["Quit DW Bar", "退出 DW Bar"],
  "關閉膠囊與背景更新。之後可從開始功能表再開啟。": ["Closes the capsule and stops background refreshes. Reopen it from the Start menu.", "关闭胶囊与后台更新。之后可从开始菜单再打开。"],
  "結束": ["Quit", "退出"],

  // ── diagnostics ──
  "複製診斷資訊": ["Copy diagnostics", "复制诊断信息"],
  "紀錄": ["Log", "日志"],
  "重新整理": ["Reload", "刷新"],
  "版本": ["Version", "版本"],
  "工作列": ["Taskbar", "任务栏"],
  "已找到": ["found", "已找到"],
  "找不到（Explorer 未就緒？）": ["not found (Explorer not ready?)", "找不到（Explorer 未就绪？）"],
  "工作列對齊": ["Taskbar alignment", "任务栏对齐"],
  "置中": ["centred", "居中"],
  "靠左": ["left", "靠左"],
  "小工具按鈕": ["Widgets button", "小组件按钮"],
  "顯示": ["shown", "显示"],
  "隱藏": ["hidden", "隐藏"],
  "浮貼（嵌入被拒，已自動改用）": ["floating (embedding refused, switched automatically)", "浮动（嵌入被拒，已自动改用）"],
  "浮貼": ["floating", "浮动"],
  "縮放": ["Scale", "缩放"],
  "工作列範圍": ["Taskbar bounds", "任务栏范围"],
  "膠囊範圍": ["Capsule bounds", "胶囊范围"],
  "全螢幕": ["Fullscreen", "全屏"],
  "是（膠囊已隱藏）": ["yes (capsule hidden)", "是（胶囊已隐藏）"],
  "否": ["no", "否"],
  "設定檔": ["Settings file", "配置文件"],
  "紀錄檔": ["Log file", "日志文件"],
  "Windows 拒絕了嵌入，目前改用「浮貼在上方」。": ["Windows refused embedding, so the capsule floats on top for now.", "Windows 拒绝了嵌入，目前改用“浮动在上方”。"],
  "（還沒有紀錄）": ["(no log yet)", "（还没有日志）"],
  "DW Bar 診斷": ["DW Bar diagnostics", "DW Bar 诊断"],
  "已複製": ["Copied", "已复制"],
  "無法存取剪貼簿": ["Couldn't access the clipboard", "无法访问剪贴板"],

  // ── usage page, panel, hover card, capsule ──
  "上次更新 {time}，每 {n} 分鐘自動更新。": ["Updated {time}, refreshes every {n} minutes.", "上次更新 {time}，每 {n} 分钟自动更新。"],
  "正在讀取各家 AI 的額度…": ["Reading each AI's limits…", "正在读取各家 AI 的额度…"],
  "正在偵測…": ["Detecting…", "正在检测…"],
  "連結你的 AI 帳號": ["Link your AI accounts", "关联你的 AI 账号"],
  "登入後，工作列就會顯示 5 小時與每週額度。登入是在官方網站上完成，DW Bar 不會看到你的密碼。": ["Once you sign in, the taskbar shows your 5-hour and weekly limits. You sign in on the official website; DW Bar never sees your password.", "登录后，任务栏就会显示 5 小时与每周额度。登录是在官方网站上完成，DW Bar 不会看到你的密码。"],
  "餘額": ["Balance", "余额"],
  "重新偵測": ["Detect again", "重新检测"],
  "重新登入": ["Sign in again", "重新登录"],
  "連結帳號": ["Link account", "关联账号"],
  "沒有資料": ["No data", "没有数据"],
  "命令列": ["command line", "命令行"],
  "方案": ["Plan", "套餐"],
  "來源": ["Source", "来源"],
  "剩 {n}%": ["{n}% left", "剩 {n}%"],
  "重置": ["Resets", "重置"],
  "{label} 剩餘": ["{label} left", "{label} 剩余"],
  "{time}（還有 {left}）": ["{time} (in {left})", "{time}（还有 {left}）"],
  "{time} 重置 · 還有 {left}": ["resets {time} · in {left}", "{time} 重置 · 还有 {left}"],
  "{left}後重置": ["resets in {left}", "{left}后重置"],
  "（{time} 重置）": [" (resets {time})", "（{time} 重置）"],
  "下次重置 {time}": ["Next reset {time}", "下次重置 {time}"],
  "點擊展開全部 AI": ["Click to show every AI", "点击展开全部 AI"],
  "點擊開啟面板": ["Click to open the panel", "点击打开面板"],
  "{n} 個 AI": ["{n} AI", "{n} 个 AI"],
  "上次更新 {time}": ["updated {time}", "上次更新 {time}"],
  "還沒有連結任何 AI。": ["No AI linked yet.", "还没有关联任何 AI。"],
  "每 {n} 分鐘自動更新": ["Refreshes every {n} min", "每 {n} 分钟自动更新"],
  "正在偵測 AI…": ["Detecting AI…", "正在检测 AI…"],
  "DW Bar：點一下連結你的 AI 帳號": ["DW Bar: click to link your AI accounts", "DW Bar：点一下关联你的 AI 账号"],
  "（點一下展開全部）": ["(click to show all)", "（点一下展开全部）"],
  "已連結 {name}": ["Linked {name}", "已关联 {name}"],
  // preview text in settings
  "3 個 AI · 上次更新 23:20": ["3 AI · updated 23:20", "3 个 AI · 上次更新 23:20"],
  "5 小時 剩餘": ["5-hour left", "5 小时 剩余"],
  "剩 66%": ["66% left", "剩 66%"],
  "每 5 分鐘自動更新": ["Refreshes every 5 min", "每 5 分钟自动更新"],

  // ── time ──
  "{d} 天 {h} 小時": ["{d} d {h} h", "{d} 天 {h} 小时"],
  "{h} 小時 {m} 分": ["{h} h {m} min", "{h} 小时 {m} 分"],
  "{m} 分": ["{m} min", "{m} 分"],

  // ── from the background engine (window labels, notices, toasts) ──
  "5 小時": ["5-hour", "5 小时"],
  "每週": ["Weekly", "每周"],
  "每週（全部模型）": ["Weekly (all models)", "每周（全部模型）"],
  "每週 · Opus": ["Weekly · Opus", "每周 · Opus"],
  "每週 · Sonnet": ["Weekly · Sonnet", "每周 · Sonnet"],
  "每日": ["Daily", "每日"],
  "每月": ["Monthly", "每月"],
  "額度": ["Quota", "额度"],
  "模型": ["Models", "模型"],
  "模型額度": ["Model quota", "模型额度"],
  "本期方案額度": ["This billing period", "本期套餐额度"],
  "無限": ["Unlimited", "无限"],
  "尚未連結帳號": ["Account not linked", "尚未关联账号"],
  "未知的 AI": ["Unknown AI", "未知的 AI"],
  "打開 Antigravity 後就會顯示用量": ["Open Antigravity to see its usage", "打开 Antigravity 后就会显示用量"],
  "找到 Antigravity，但它的額度服務沒有回應": ["Found Antigravity, but its quota service didn't respond", "找到 Antigravity，但它的额度服务没有响应"],
  "Antigravity 正在啟動，稍後自動重試": ["Antigravity is starting; retrying shortly", "Antigravity 正在启动，稍后自动重试"],
  "Antigravity 用量目前只支援 Windows 版": ["Antigravity usage is Windows-only for now", "Antigravity 用量目前只支持 Windows 版"],
  "沒有輸出": ["no output", "没有输出"],
  "Antigravity 沒有在 30 秒內回應": ["Antigravity didn't respond within 30 seconds", "Antigravity 没有在 30 秒内响应"],
  "Antigravity 回傳了無法辨識的資料": ["Antigravity returned data DW Bar can't read", "Antigravity 返回了无法识别的数据"],
  "Claude Code 沒有在 40 秒內回覆用量": ["Claude Code didn't report usage within 40 seconds", "Claude Code 没有在 40 秒内返回用量"],
  "此登入方式沒有訂閱額度（API key、第三方平台，或尚未登入）": ["This sign-in has no subscription limits (API key, third-party platform, or not signed in)", "此登录方式没有订阅额度（API key、第三方平台，或尚未登录）"],
  "這次沒有讀到 Claude 額度，稍後自動重試": ["Couldn't read Claude's limits this time; retrying shortly", "这次没有读到 Claude 额度，稍后自动重试"],
  "Claude 回傳了無法辨識的額度資料": ["Claude returned limits DW Bar can't read", "Claude 返回了无法识别的额度数据"],
  "Codex 尚未用 ChatGPT 帳號登入（執行 codex login）": ["Codex isn't signed in with a ChatGPT account (run codex login)", "Codex 尚未用 ChatGPT 账号登录（运行 codex login）"],
  "此登入方式（API key）沒有訂閱額度": ["This sign-in (API key) has no subscription limits", "此登录方式（API key）没有订阅额度"],
  "Codex 沒有在 30 秒內回覆用量": ["Codex didn't report usage within 30 seconds", "Codex 没有在 30 秒内返回用量"],
  "Codex 沒有回傳額度資料": ["Codex didn't return any limits", "Codex 没有返回额度数据"],
  "Codex 回傳了無法辨識的額度資料": ["Codex returned limits DW Bar can't read", "Codex 返回了无法识别的额度数据"],
  "沒有 stdout": ["no stdout", "没有 stdout"],
  "stdin 已關閉": ["stdin closed", "stdin 已关闭"],
  "這個 Google 帳號沒有 Gemini 用量資料": ["This Google account has no Gemini usage data", "这个 Google 账号没有 Gemini 用量数据"],
  "Cursor 回傳了無法辨識的用量資料": ["Cursor returned usage DW Bar can't read", "Cursor 返回了无法识别的用量数据"],
  "Windsurf 沒有回傳額度（可能是試用或團隊方案）": ["Windsurf returned no limits (maybe a trial or team plan)", "Windsurf 没有返回额度（可能是试用或团队套餐）"],
  "DeepSeek 沒有回傳餘額": ["DeepSeek returned no balance", "DeepSeek 没有返回余额"],
  "ChatGPT 帳號沒有 Codex 額度資料（可能是免費方案）": ["This ChatGPT account has no Codex limits (maybe the free plan)", "ChatGPT 账号没有 Codex 额度数据（可能是免费套餐）"],
  "網站沒有回應": ["The website didn't respond", "网站没有响应"],
  "沒有回應": ["no response", "没有响应"],
  "已顯示小工具": ["Widgets shown", "已显示小组件"],
  "已關閉小工具（天氣）": ["Widgets (weather) turned off", "已关闭小组件（天气）"],
  "Windows 不允許其他程式更改這個設定，已幫你打開「工作列設定」，請把「小工具」關掉。": ["Windows doesn't let other apps change this, so Taskbar settings is open for you: turn off “Widgets” there.", "Windows 不允许其他程序更改这个设置，已帮你打开“任务栏设置”，请把“小组件”关掉。"],
  "只支援 Windows": ["Windows only", "只支持 Windows"],
  "Windows 不允許其他程式更改這個設定，已幫你打開「工作列設定」，請把「小工具」打開。": ["Windows doesn't let other apps change this, so Taskbar settings is open for you: turn on “Widgets” there.", "Windows 不允许其他程序更改这个设置，已帮你打开“任务栏设置”，请把“小组件”打开。"],
};

// Engine messages with a variable part.
const PATTERNS = [
  [/^讀取 Antigravity 失敗：(.*)$/s, ["Couldn't read Antigravity: $1", "读取 Antigravity 失败：$1"]],
  [/^無法啟動 PowerShell：(.*)$/s, ["Couldn't start PowerShell: $1", "无法启动 PowerShell：$1"]],
  [/^(.+) · 5 小時$/, ["$1 · 5-hour", "$1 · 5 小时"]],
  [/^(.+) · 每週$/, ["$1 · Weekly", "$1 · 每周"]],
  [/^Claude Code 查詢失敗：(.*)$/s, ["Claude Code query failed: $1", "Claude Code 查询失败：$1"]],
  [/^(\d+) 小時$/, ["$1-hour", "$1 小时"]],
  [/^無法啟動 (.+?)：(.*)$/s, ["Couldn't start $1: $2", "无法启动 $1：$2"]],
  [/^寫入失敗：(.*)$/s, ["Write failed: $1", "写入失败：$1"]],
  [/^按量剩 (.+)$/, ["$1 on-demand left", "按量剩 $1"]],
  [/^額外用量剩 (.+)$/, ["$1 extra usage left", "额外用量剩 $1"]],
  [/^無法建立讀取視窗：(.*)$/s, ["Couldn't open the reader window: $1", "无法创建读取窗口：$1"]],
  [/^(.+) 登入已過期，請重新連結帳號$/, ["$1 sign-in expired; please link the account again", "$1 登录已过期，请重新关联账号"]],
  [/^暫時讀不到用量（(.*)），稍後自動重試$/s, ["Can't read usage right now ($1); retrying shortly", "暂时读不到用量（$1），稍后自动重试"]],
  [/^(.*)。也可以直接連結帳號。$/s, ["$1. You can also link the account instead.", "$1。也可以直接关联账号。"]],
  [/^無法儲存設定：(.*)$/s, ["Couldn't save settings: $1", "无法保存设置：$1"]],
];

export let lang = "zh-TW";

/** "auto" → the system / browser language: Traditional Chinese, Simplified Chinese or English. */
export function resolveLang(setting) {
  if (setting === "zh-TW" || setting === "zh-CN" || setting === "en") return setting;
  for (const l of navigator.languages?.length ? navigator.languages : [navigator.language || "en"]) {
    const s = String(l).toLowerCase();
    if (s.startsWith("zh")) return /hant|tw|hk|mo/.test(s) ? "zh-TW" : "zh-CN";
    if (s) return "en";
  }
  return "en";
}

export function setLang(setting) {
  lang = resolveLang(setting);
  document.documentElement.lang = lang === "en" ? "en" : lang;
  return lang;
}

/** Locale for dates and numbers. */
export const locale = () => (lang === "en" ? "en-US" : lang);

const fill = (s, vars) => (vars ? s.replace(/\{(\w+)\}/g, (m, k) => (k in vars ? vars[k] : m)) : s);

/** Translate a UI string (Traditional Chinese key). */
export function t(key, vars) {
  if (lang === "zh-TW") return fill(key, vars);
  const e = D[key];
  return fill(e ? e[lang === "en" ? 0 : 1] : key, vars);
}

/** Translate text that came from the background engine. Unknown text is shown as is. */
export function tb(text) {
  if (text == null || lang === "zh-TW") return text;
  const s = String(text);
  const e = D[s];
  if (e) return e[lang === "en" ? 0 : 1];
  for (const [re, out] of PATTERNS) {
    if (re.test(s)) return s.replace(re, out[lang === "en" ? 0 : 1]);
  }
  return s;
}

// Static HTML: remember the original text of every node once, translate on each language change.
let originals = null;
const ATTRS = ["title", "aria-label", "placeholder"];
export function localizePage(root = document) {
  if (!originals) {
    originals = [];
    const walker = document.createTreeWalker(root.body || root, NodeFilter.SHOW_TEXT);
    for (let n = walker.nextNode(); n; n = walker.nextNode()) {
      if (n.nodeValue.trim() && !n.parentElement?.closest("script,style,[data-no-i18n]")) originals.push([n, null, n.nodeValue]);
    }
    for (const el of (root.body || root).querySelectorAll(ATTRS.map((a) => `[${a}]`).join(","))) {
      for (const a of ATTRS) if (el.hasAttribute(a)) originals.push([el, a, el.getAttribute(a)]);
    }
    originals.push([document, "title", document.title]);
  }
  for (const [node, attr, text] of originals) {
    const trimmed = text.trim();
    const out = text.replace(trimmed, t(trimmed));
    if (attr === "title" && node === document) document.title = out;
    else if (attr) node.setAttribute(attr, out);
    else node.nodeValue = out;
  }
}
