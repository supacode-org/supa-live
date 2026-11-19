# 錄製或合併時阻止系統睡眠

## 摘要

只要有任何一個直播間在錄製中或合併中，就阻止系統閒置睡眠，避免長時間沒人操作時電腦睡著、錄製中斷。這個功能預設開啟，可以在設定裡關掉。

## 行為

- 「忙碌」的定義：任何一次錄製的狀態是錄製中（`rec`）或合併中（`merging`），包含手動合併和錄製結束後的自動合併。
- 從不忙碌變成忙碌時向系統要求不睡眠；全部結束、或使用者關掉設定時立刻放掉。
- 只阻止**閒置**造成的系統睡眠：
  - 螢幕照常可以關閉或進入螢幕保護程式；
  - 使用者主動選「睡眠」、闔上筆電螢幕（沒接外接螢幕時）、電量過低，系統照樣會睡。
- 程式結束或當掉時，系統會自動收回要求，不會殘留。

## 實作

不加新依賴，直接呼叫系統 API。放在桌面版 `awake.rs`。

| 平台 | 做法 |
| --- | --- |
| macOS | IOKit `IOPMAssertionCreateWithName`，類型 `PreventUserIdleSystemSleep`，結束時 `IOPMAssertionRelease`。屬於公開 API，上架版（關掉 `macOSPrivateApi`）也能用 |
| Windows | `PowerCreateRequest` + `PowerSetRequest(PowerRequestSystemRequired)`，結束時 `PowerClearRequest` 並關掉 handle。不綁定執行緒，和 `SetThreadExecutionState` 不同 |
| 其他 | 不做事 |

- 說明文字（macOS `pmset -g assertions`、Windows `powercfg /requests` 看得到）是「Supa Live 正在錄製或合併」，跟著介面語言。
- 每次錄製狀態或設定改變時重新判斷一次，狀態沒變就不重複呼叫系統。
- 手動合併時先確認還有分段可以合併，才把狀態標成合併中。否則找不到分段時會一直停在合併中，電腦也就一直不睡。

## 設定

- `supalive.json` 的 `settings.keepAwake`，布林，預設 `true`。舊的設定檔沒有這個欄位時當成 `true`。
- 設定畫面加一個開關「錄製或合併時不讓電腦睡眠」。

## 驗證

- 單元測試：只有錄製中或合併中算忙碌，全部完成不算；設定關掉時不算。
- macOS：開始錄製後 `pmset -g assertions` 看得到 Supa Live 的 `PreventUserIdleSystemSleep`；停止並合併完成後消失；錄製中關掉設定也會消失。（需要真實直播，只能在 Leo 的 Mac 上測。）
- Windows：同上，用系統管理員身分執行 `powercfg /requests`，在 SYSTEM 底下看得到。

## 不在範圍內

- 阻止螢幕關閉。
- 闔上筆電螢幕時仍保持運作。
- 命令列版本。
