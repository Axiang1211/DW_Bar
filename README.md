# DW Bar

把 AI 的額度放在 Windows 工作列上的小膠囊。

外圈上半弧是 **5 小時額度**，內圈是 **每週額度**，綠／黃／紅代表剩多少。滑鼠停在圖示上看詳細，點一下開設定。

> 目前是 **MVP 骨架（v0.1）**：工作列嵌入與定位、Claude Code／Codex 額度、設定視窗、全螢幕自動隱藏。

## 下載安裝（不用裝任何開發工具）

每次推送程式碼到 `main`，GitHub 會自動在雲端的 Windows 電腦上打包安裝檔：

1. 打開本專案的 **Actions** 分頁
2. 點最上面一筆綠色勾勾的 **Build**
3. 往下捲到 **Artifacts**，下載 **DW-Bar-Windows**（zip）
4. 解壓縮，執行 `DW Bar_0.1.0_x64-setup.exe`（或 `.msi`）

第一次執行，Windows SmartScreen 可能會提示「無法辨識的應用程式」，因為安裝檔還沒有數位簽章：按「其他資訊 → 仍要執行」。

## 功能（v0.1）

| 項目 | 說明 |
|---|---|
| 工作列嵌入 | 膠囊成為 `Shell_TrayWnd` 的子視窗，跟著工作列移動、自動隱藏。被拒絕時自動改用「浮貼」。 |
| 定位規則 | 自動：工作列置中 → 最左邊（同 Windows 小工具，並讓出小工具的位置）；靠左 → 系統匣左側。可微調像素。 |
| Explorer 重啟 | 監聽 `TaskbarCreated`，膠囊自動重建。開機 2.5 秒後再對位一次。 |
| 全螢幕隱藏 | `SHQueryUserNotificationState` ＋ 前景視窗蓋滿螢幕判斷，300 ms 防抖。 |
| Claude Code 額度 | 透過 `claude` 自己的 `get_usage`，不讀取任何 token 或 cookie。 |
| Codex 額度 | 透過 `codex app-server` 的 `account/rateLimits/read`。 |
| 設定視窗 | 用量、位置、外觀、AI 偵測（含自訂路徑）、一般（開機啟動、更新頻率）、診斷（紀錄與狀態）。 |

偵測不到的 AI 不會出現在工作列。設定存在 `%LOCALAPPDATA%\DWBar\config.json`，紀錄在 `%LOCALAPPDATA%\DWBar\logs\`。

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
  src/quota/           Claude Code、Codex 額度讀取
  src/win/             Win32：找工作列、嵌入、全螢幕偵測、監看執行緒
src-tauri/             Tauri v2 外殼：視窗、系統匣圖示、指令
index.html             工作列膠囊
settings.html          設定視窗
.github/workflows/     GitHub Actions：Windows 打包
```

執行緒規則：監看執行緒只負責「偵測」，所有視窗操作都回到 UI 執行緒做。

## 接下來

- 即時狀態（工作中／等待你回覆）：`dwbar-status.exe` hook ＋ 狀態檔
- 點擊後的彈出面板、Auto-Resume、紙飛機協作動畫
- 多螢幕、最大化時加深工作列、遊戲排除名單

## 致謝

- 額度讀取的做法參考 [Agent HUD Open](https://github.com/jazzenchen/agent-hud-open)（Apache-2.0），本專案為獨立實作。
- 部分方向參考 ZenDesktop 的公開說明，未使用其程式碼。
