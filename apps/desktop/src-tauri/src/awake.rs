//! 有直播間在錄製或合併時，阻止系統閒置睡眠。只擋閒置睡眠，螢幕照常可以關。
//! 直接呼叫系統 API，不加依賴；程式結束或當掉時系統會自動收回要求。

use std::sync::Mutex;

use supa_live_core::tr;

use crate::store::{Data, RecStatus};

/// 設定開著，而且有任何一次錄製還在錄製中或合併中。
pub fn wanted(d: &Data) -> bool {
    d.settings.keep_awake && d.rooms.iter().flat_map(|r| &r.recordings).any(|x| x.status != RecStatus::Done)
}

/// 目前向系統要求的狀態。狀態沒變就不重複呼叫系統。
#[derive(Default)]
pub struct Awake(Mutex<Option<sys::Guard>>);

impl Awake {
    pub fn set(&self, on: bool) {
        let mut g = self.0.lock().unwrap();
        if on == g.is_some() {
            return;
        }
        // 要求失敗時維持 None，下次狀態變動會再試
        *g = if on {
            sys::Guard::new(&tr!("Supa Live 正在錄製或合併", "Supa Live is recording or merging"))
        } else {
            None
        };
    }
}

#[cfg(target_os = "macos")]
mod sys {
    use std::ffi::c_void;

    type CFStringRef = *const c_void;

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithBytes(
            alloc: *const c_void,
            bytes: *const u8,
            len: isize,
            encoding: u32,
            external: u8,
        ) -> CFStringRef;
        fn CFRelease(cf: *const c_void);
    }

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOPMAssertionCreateWithName(kind: CFStringRef, level: u32, name: CFStringRef, id: *mut u32) -> i32;
        fn IOPMAssertionRelease(id: u32) -> i32;
    }

    const UTF8: u32 = 0x0800_0100;
    const LEVEL_ON: u32 = 255;

    fn cfstr(s: &str) -> CFStringRef {
        // SAFETY: bytes 與長度來自同一個 &str，CoreFoundation 會複製內容
        unsafe { CFStringCreateWithBytes(std::ptr::null(), s.as_ptr(), s.len() as isize, UTF8, 0) }
    }

    pub struct Guard(u32);

    impl Guard {
        pub fn new(reason: &str) -> Option<Self> {
            let kind = cfstr("PreventUserIdleSystemSleep");
            let name = cfstr(reason);
            if kind.is_null() || name.is_null() {
                // SAFETY: 只釋放建立成功的字串
                unsafe {
                    for s in [kind, name].into_iter().filter(|s| !s.is_null()) {
                        CFRelease(s);
                    }
                }
                return None;
            }
            let mut id = 0;
            // SAFETY: 兩個字串都有效，id 指向本地變數；IOKit 會自己保留字串，呼叫後可以釋放
            let ok = unsafe {
                let r = IOPMAssertionCreateWithName(kind, LEVEL_ON, name, &mut id);
                CFRelease(kind);
                CFRelease(name);
                r == 0
            };
            ok.then_some(Self(id))
        }
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            // SAFETY: id 是建立成功時拿到的，只釋放一次
            unsafe {
                IOPMAssertionRelease(self.0);
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod sys {
    use std::ffi::c_void;

    /// REASON_CONTEXT：Reason 是聯集，最大的成員是 { HMODULE, ULONG, LPWSTR* }，補齊到同樣大小。
    #[repr(C)]
    struct ReasonContext {
        version: u32,
        flags: u32,
        simple: *const u16,
        _rest: [usize; 2],
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn PowerCreateRequest(ctx: *const ReasonContext) -> *mut c_void;
        fn PowerSetRequest(h: *mut c_void, kind: i32) -> i32;
        fn PowerClearRequest(h: *mut c_void, kind: i32) -> i32;
        fn CloseHandle(h: *mut c_void) -> i32;
    }

    const POWER_REQUEST_CONTEXT_SIMPLE_STRING: u32 = 0x1;
    const POWER_REQUEST_SYSTEM_REQUIRED: i32 = 1;
    const INVALID_HANDLE_VALUE: *mut c_void = -1isize as *mut c_void;

    pub struct Guard(*mut c_void);

    // SAFETY: 電源要求的 handle 不綁定執行緒，可以在任何執行緒清除和關閉
    unsafe impl Send for Guard {}

    impl Guard {
        pub fn new(reason: &str) -> Option<Self> {
            let wide: Vec<u16> = reason.encode_utf16().chain([0]).collect();
            let ctx = ReasonContext {
                version: 0,
                flags: POWER_REQUEST_CONTEXT_SIMPLE_STRING,
                simple: wide.as_ptr(),
                _rest: [0; 2],
            };
            // SAFETY: ctx 與 wide 在呼叫期間有效，系統會複製說明文字
            unsafe {
                let h = PowerCreateRequest(&ctx);
                if h.is_null() || h == INVALID_HANDLE_VALUE {
                    return None;
                }
                if PowerSetRequest(h, POWER_REQUEST_SYSTEM_REQUIRED) == 0 {
                    CloseHandle(h);
                    return None;
                }
                Some(Self(h))
            }
        }
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            // SAFETY: handle 是建立並設定成功時拿到的，只清除和關閉一次
            unsafe {
                PowerClearRequest(self.0, POWER_REQUEST_SYSTEM_REQUIRED);
                CloseHandle(self.0);
            }
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod sys {
    pub struct Guard;

    impl Guard {
        pub fn new(_reason: &str) -> Option<Self> {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{Recording, Room};

    fn data(statuses: &[RecStatus]) -> Data {
        let recordings = statuses.iter().map(|s| Recording { status: s.clone(), ..Default::default() }).collect();
        Data { rooms: vec![Room { recordings, ..Default::default() }], ..Default::default() }
    }

    #[test]
    fn busy_when_recording_or_merging() {
        assert!(wanted(&data(&[RecStatus::Done, RecStatus::Rec])));
        assert!(wanted(&data(&[RecStatus::Merging])));
    }

    #[test]
    fn idle_when_all_done() {
        assert!(!wanted(&data(&[RecStatus::Done, RecStatus::Done])));
        assert!(!wanted(&Data::default()));
    }

    #[test]
    fn off_when_setting_disabled() {
        let mut d = data(&[RecStatus::Rec]);
        d.settings.keep_awake = false;
        assert!(!wanted(&d));
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    #[test]
    fn system_accepts_request() {
        let g = sys::Guard::new("Supa Live test");
        assert!(g.is_some());
    }

    #[test]
    fn defaults_on_for_old_settings() {
        let d: Data = serde_json::from_str(r#"{"settings":{"segMin":30}}"#).unwrap();
        assert!(d.settings.keep_awake);
    }
}
