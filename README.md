<div align="center">
  <img src="docs/assets/icon.png" alt="Supa Live" width="128" />
  <h1>Supa Live</h1>
  <p>抖音直播錄製工具。保留原始畫質，無二次轉碼，免裝 FFmpeg，開箱即用。</p>
  <p>
    <img alt="Platforms" src="https://img.shields.io/badge/platforms-macOS%20%7C%20Windows-blue.svg" />
    <img alt="License" src="https://img.shields.io/badge/license-GPL--3.0-green.svg" />
  </p>
</div>

## 功能

- **保留原畫質**：無二次轉碼，直接存成 MP4，支援 H.264 與 HEVC。
- **免裝 FFmpeg**：不依賴外部工具，開箱即用。
- **貼上即錄**：貼上直播間網址或抖音 App 的分享文字，自動列出可用畫質。
- **同時錄多台**：支援同時錄製多個直播間，彼此獨立不干擾。
- **邊錄邊看**：錄製中可直接預覽，支援往回拉看最近約 3 分鐘，並顯示即時碼率與幀率。
- **斷線不掉檔**：斷線自動重連，錄完自動合併；就算中途當機或斷電，已錄好的內容也不會損壞。
- **減少硬碟寫入**：錄製資料會先暫存在記憶體再批次寫入，緩衝大小可自訂。
- **自動整理檔案**：依直播間與時間自動分類並保存封面，刪除時預設移到廢紙簍。

## 下載與安裝

### 直接下載

到 [GitHub Releases](https://github.com/supacode-org/supa-live/releases) 下載對應系統的安裝檔（支援 macOS / Windows，arm64 與 x86_64 皆有）。

### 自行建置

需要 Rust 1.88 以上與 Tauri CLI：

```sh
cargo install tauri-cli --version "^2" --locked
cd apps/desktop/src-tauri
cargo tauri build
```

進階說明與命令列版本請見 [開發文件](docs/DEVELOPMENT.md)。

## 怎麼用

1. 點「新增直播間」，貼上直播間網址或分享文字。
2. 在右側選好畫質，點「錄製這個畫質」開始錄製。
3. 檔案預設存在 `下載/Supa Live/<直播間名>/<錄製時間>/`。

如果拉流連結過期，點一下「重新抓取連結」就能繼續。

## 常見問題

**Windows 上無法在 App 內預覽 HEVC 直播？**
到 Microsoft Store 安裝「HEVC 視訊延伸模組」即可。不裝也不影響錄製，錄下來的影片一樣可以用其他播放器看。

**最多可以同時錄幾個直播間？**
軟體本身不設上限，但同時錄超過 2 個容易觸發抖音風控，導致暫時被限流。如果開始頻繁斷線，建議減少同時錄製的數量。

**有支援 Linux 嗎？**
目前正式支援 macOS 與 Windows。Linux 雖然能編譯執行，但還有部分功能不穩定。

## 免責聲明

本專案僅供學習與個人使用。請只錄製你有權保存的內容，並遵守抖音服務條款與當地法規。

## 授權

以 [GPL-3.0-or-later](LICENSE) 授權釋出。
