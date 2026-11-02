<div align="center">
  <img src="docs/assets/icon.png" alt="SupaLive" width="128" />
  <h1>SupaLive</h1>
  <p>抖音直播錄製工具，直接保存原始串流，畫質與直播完全一致。</p>
  <p>
    <img alt="Platforms" src="https://img.shields.io/badge/platforms-macOS%20%7C%20Windows-blue.svg" />
    <img alt="License" src="https://img.shields.io/badge/license-GPL--3.0-green.svg" />
  </p>
</div>

## 簡介

SupaLive 是一款桌面端的抖音直播錄製工具。它直接保存直播間的原始串流，錄下來的畫質與直播一致。封裝功能完全自行實作，安裝後即可使用。

## 功能特性

- **貼網址錄製**：貼上直播間網址或抖音 App 的分享文字，自動抓取各種畫質供選擇。
- **原畫質保存**：把原始串流直接封裝成 MP4，保留直播原畫質，支援 H.264 與 HEVC。
- **多路同時錄製**：多個直播間同時錄製，各路獨立運作；同時超過 2 路時提醒風控風險。
- **邊錄邊看**：錄製中可直接播放，支援回看最近約 3 分鐘，並顯示即時碼率與幀率。
- **分段保存**：存成分段式 MP4，錄製中斷時已寫入的內容完整保留。
- **斷線重連、自動合併**：斷線後自動重連，錄製結束後自動合併分段。
- **降低磁碟寫入**：資料先緩存在記憶體，緩衝大小可調。
- **錄製管理**：按直播間、錄製時間分層整理，自動保存封面；刪除時預設移到廢紙簍。
- **多語言**：支援正體中文與英文，預設跟隨系統。

## 快速開始

### 下載

到 [GitHub Releases](https://github.com/supacode-org/supalive/releases) 下載 macOS 或 Windows 的安裝檔（arm64、x86_64 皆有）。

### 從原始碼建置

需要 Rust 1.88 以上，以及 Tauri 的命令列工具：

```sh
cargo install tauri-cli --version "^2" --locked
cd apps/desktop/src-tauri
cargo tauri build
```

更多說明（包括命令列版）見[開發文件](docs/DEVELOPMENT.md)。

## 使用方法

1. 點擊「新增直播間」，貼上直播間網址。
2. 在右側「選取直播」中選擇畫質，點擊「錄製這個畫質」。
3. 錄製的檔案預設保存在 `下載/SupaLive/<直播間名>/<錄製時間>/`。

連結過期後，點擊「重新抓取連結」即可重新取得。

## 常見問題

**Windows 上怎麼在 App 內播放錄製中的 HEVC 直播？**

安裝「HEVC 視訊延伸模組」即可。錄製照常進行，檔案也可以用其他播放器打開。

**同時錄製多少個直播間比較安全？**

程式本身支援任意數量，但同時超過 2 個可能觸發抖音風控，導致拉流被暫時封鎖。如果開始頻繁斷線，請減少同時錄製的數量。

**支援 Linux 嗎？**

目前正式支援 macOS 與 Windows。Linux 可以編譯執行，部分功能仍有問題。

## 免責聲明

本專案僅供學習與個人使用。請只錄製你有權保存的內容，並遵守抖音的使用條款與相關法律法規。

## 授權

本專案以 [GPL-3.0-or-later](LICENSE) 授權。
