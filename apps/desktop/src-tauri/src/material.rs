//! 視窗層的原生材質：這是兩個平台唯一刻意不同的地方，視窗內的介面完全共用。

use tauri::WebviewWindow;

/// 回傳實際套用的材質，介面據此決定要不要自己鋪底色。
/// `transparent_ok` 為 false（例如上架版關掉 macOSPrivateApi）時一律不套用。
pub fn apply(win: &WebviewWindow, transparent_ok: bool) -> &'static str {
    #[cfg(target_os = "macos")]
    if transparent_ok {
        use window_vibrancy::{apply_liquid_glass, apply_vibrancy, LiquidGlassOptions, NSVisualEffectMaterial};
        // macOS 26 以上用液態玻璃，舊版退回 vibrancy
        if apply_liquid_glass(win, LiquidGlassOptions::default()).is_ok() {
            return "liquid-glass";
        }
        if apply_vibrancy(win, NSVisualEffectMaterial::Sidebar, None, None).is_ok() {
            return "vibrancy";
        }
    }
    #[cfg(target_os = "windows")]
    {
        let _ = transparent_ok;
        round_corners(win);
        // Windows 11 才有 Mica；Windows 10 會失敗，介面改用純色底
        if window_vibrancy::apply_mica(win, None).is_ok() {
            return "mica";
        }
    }
    let _ = (win, transparent_ok);
    "none"
}

/// Windows 11 主動要求圓角，透明視窗也不會變成直角。Windows 10 沒有這個屬性，失敗就維持直角。
/// 沒有硬體加速時（例如沒有 GPU 驅動的虛擬機）系統會關掉所有圓角，這裡改變不了。
#[cfg(target_os = "windows")]
fn round_corners(win: &WebviewWindow) {
    use std::ffi::c_void;

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(hwnd: *mut c_void, attr: u32, value: *const c_void, size: u32) -> i32;
    }
    const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
    const DWMWCP_ROUND: u32 = 2;

    let Ok(hwnd) = win.hwnd() else { return };
    let pref = DWMWCP_ROUND;
    // SAFETY: hwnd 是這個視窗自己的 handle，pref 在呼叫期間有效，大小與 DWM 要求的 DWORD 相同
    unsafe {
        DwmSetWindowAttribute(
            hwnd.0,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            (&pref as *const u32).cast(),
            std::mem::size_of::<u32>() as u32,
        );
    }
}
