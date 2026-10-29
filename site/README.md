# dwbar.app.dreamsworld.top

DW Bar 的官方網站（靜態頁面 + Cloudflare Pages Functions）。

- `index.html` 介紹、使用方式、安裝須知、常見問題、下載
- `privacy.html`、`terms.html` 隱私權政策、服務條款
- `functions/download.js` `/download`：直接串流最新版安裝檔（GitHub Release 的 `DW-Bar-Setup.exe`），網址不離開本站
- `functions/api/latest.js` `/api/latest`：目前版本與檔案大小

## 部署（Cloudflare Pages，跟其他 app.dreamsworld.top 子網站一樣）

1. Cloudflare 後台 → Workers & Pages → 建立 → Pages → 連接 Git → 選 `Axiang1211/DW_Bar`
2. 建置設定：框架「無」、建置命令留空、**根目錄 `site`**、輸出目錄 `/`
3. 部署完成後 → 自訂網域 → 新增 `dwbar.app.dreamsworld.top`（網域在 Cloudflare 上會自動加 DNS）

之後每次推送到 `main`：GitHub Actions 會打包並發布新版本，網站的下載按鈕自動提供最新版。
