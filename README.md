# DW Bar

把 AI 的額度放在 Windows 工作列上的小膠囊。

外圈上半弧是 **5 小時額度**，內圈是 **每週額度**，綠／黃／紅代表剩多少。滑鼠移到圖示上會跳出用量小卡（5 小時、每週、餘額），點一下開設定。

> 目前版本 **v0.7**：新標誌、官方網站 [dwbar.app.dreamsworld.top](https://dwbar.app.dreamsworld.top)、半透明視窗開關；各家官方圖示（4 種風格）、滑鼠移上去的用量小卡、點一下開啟用量面板、九宮格位置；支援 Claude、Codex（ChatGPT）、Gemini、Antigravity、Cursor、Windsurf、DeepSeek（DSH）。自動偵測已安裝的 App，在 DW Bar 內連結帳號即可顯示用量。

## 下載安裝（不用裝任何開發工具）

到 **[dwbar.app.dreamsworld.top](https://dwbar.app.dreamsworld.top)** 按「下載」，或在本專案的 **Releases** 下載 `DW-Bar-Setup.exe`。

1. 執行 `DW-Bar-Setup.exe`（或 `DW-Bar.msi`）
2. 打開設定 →「AI 帳號」→ 按想要的 AI 的「連結」按鈕，在跳出的官方登入頁登入

不需要安裝任何命令列工具。

第一次執行，Windows SmartScreen 可能會提示「無法辨識的應用程式」，因為安裝檔還沒有數位簽章：按「其他資訊 → 仍要執行」。

## 額度從哪裡來

每個 AI 依序嘗試：

1. **在 DW Bar 內連結的帳號**：登入在官方網站上完成，Google／Apple／Microsoft 登入都可以；DW Bar 不會看到密碼，登入狀態只存在這台電腦的 DW Bar 裡。之後由隱藏視窗在同一個網站上讀取官方網頁版自己使用的用量資料。
2. **命令列工具或桌面 App 內建的引擎**（Claude、Codex）：有裝 `claude`／`codex`，或 Claude、ChatGPT／Codex 桌面 App 內附的引擎時自動使用。

| AI | 連結的網站 | 顯示 |
|---|---|---|
| Claude | claude.ai | 5 小時、每週 |
| Codex | chatgpt.com | 5 小時、每週（Codex 額度） |
| Gemini | gemini.google.com | 5 小時、每週 |
| Antigravity | 不需登入：打開 Antigravity 時讀取它本機的額度服務 | 各模型群組的 5 小時、每週 |
| Cursor | cursor.com | 本期方案額度、按量餘額 |
| Windsurf | windsurf.com | 每日、每週 |
| DeepSeek（DSH） | platform.deepseek.com | API 餘額 |

官網的用量資料不是公開 API，網站改版時可能需要更新 DW Bar。

## 功能

| 項目 | 說明 |
|---|---|
| 工作列嵌入 | 膠囊成為 `Shell_TrayWnd` 的子視窗，跟著工作列移動、自動隱藏。被拒絕時自動改用「浮貼」。 |
| 九宮格位置 | 中間＝嵌在工作列；其他八格＝懸浮膠囊（左右直向、上下橫向；四個角平常只顯示一個 AI，點一下展開，2 分鐘沒操作收回）。 |
| 工作列位置規則 | 小工具位置（建議）：工作列置中 → 最左邊；靠左 → 系統匣左側。也可固定最左或固定系統匣左邊，並可微調像素。 |
| Explorer 重啟 | 監聽 `TaskbarCreated`，膠囊自動重建。開機 2.5 秒後再對位一次。 |
| 全螢幕隱藏 | `SHQueryUserNotificationState` ＋ 前景視窗蓋滿螢幕判斷，300 ms 防抖。 |
| Claude 額度 | 連結的 claude.ai 帳號，或 `claude` 的 `get_usage`。 |
| Codex 額度 | 連結的 ChatGPT 帳號，或 `codex app-server` 的 `account/rateLimits/read`。 |
| 桌面 App 偵測 | 掃描程式資料夾、Microsoft Store 套件與開始功能表捷徑（也包含從 Chrome／Edge 安裝的網頁 App）。 |
| 圖示 | 圈圈中間顯示各家官方圖示（執行時從官網載入，不隨 DW Bar 散布），風格：極簡線條、賽博霓虹、毛玻璃、像素風、原色。 |
| 用量面板 | 點圈圈開啟：每個 AI 一張卡片，大圈＋剩餘百分比、每週、重置時間、餘額；可立即更新、連結帳號。 |
| 半透明視窗 | 一個開關：用 Windows 公開的 DWM 視窗屬性讓檔案總管與其他視窗的標題列變成亞克力半透明。不注入程式碼，關閉或結束 DW Bar 時還原。預設關閉。 |
| 設定視窗 | 用量、位置、外觀、AI 帳號、一般（開機啟動、更新頻率）、診斷（紀錄與狀態）。 |

沒有桌面 App、命令列工具，也沒有連結帳號的 AI 不會出現在工作列。設定存在 `%LOCALAPPDATA%\DWBar\config.json`，紀錄在 `%LOCALAPPDATA%\DWBar\logs\`。

## 自己編譯（選用）

1. 下載並雙擊 [`scripts/setup-dev-tools.bat`](scripts/setup-dev-tools.bat)，裝好 VS Build Tools、Rust、Node.js、GitHub Desktop，然後重新開機
2. 用 GitHub Desktop 把本專案 Clone 下來
3. 在專案資料夾開終端機：

```bat
npm install
npm run tauri dev
npm run tauri build
```

`tauri dev` 是開發模式，`tauri build` 會打包安裝檔到 `target\release\bundle\`。

## 專案結構

```
crates/dwbar-core/     不依賴 WebView 的核心（可在任何系統跑單元測試）
  src/layout.rs        膠囊位置計算（純函式）
  src/quota/           額度讀取（命令列、帳號用量解析）
  src/apps.rs          桌面 App 與內建引擎偵測
  src/win/             Win32：找工作列、嵌入、全螢幕偵測、監看執行緒
src-tauri/             Tauri v2 外殼：視窗、系統匣圖示、指令、帳號登入（accounts.rs）
index.html             工作列膠囊
settings.html          設定視窗
site/                  官方網站 dwbar.app.dreamsworld.top（Cloudflare Pages）
.github/workflows/     GitHub Actions：Windows 打包與發布 Release
```

執行緒規則：監看執行緒只負責「偵測」，所有視窗操作都回到 UI 執行緒做。

## 接下來

- 即時狀態（工作中／等待你回覆）：`dwbar-status.exe` hook ＋ 狀態檔
- 點擊後的彈出面板、Auto-Resume、紙飛機協作動畫
- 多螢幕、最大化時加深工作列、遊戲排除名單

## 標誌

DW Bar 的標誌以 Dreamsworld 的角色為主體，外圍加上 DW Bar 的額度計量表（上半弧＋星星指針）。

## 致謝

- 額度讀取的做法參考 [Agent HUD Open](https://github.com/jazzenchen/agent-hud-open)（Apache-2.0）與 [CodexBar](https://github.com/steipete/CodexBar)（MIT），本專案為獨立實作。
- 部分方向參考 ZenDesktop 的公開說明，未使用其程式碼。
