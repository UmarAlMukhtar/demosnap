#![allow(dead_code)]

use serde::Serialize;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, HHOOK, MSG, MSLLHOOKSTRUCT, WH_MOUSE_LL, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEMOVE,
};

// This module defines the structure for input events captured during a recording session.
#[derive(Debug, Clone, Serialize)]
pub struct ClickEvent {
    pub timestamp_ms: u64,          // Timestamp in milliseconds since UNIX epoch
    pub x: i32,                     // X coordinate of the event
    pub y: i32,                     // Y coordinate of the event
    pub event_type: MouseEventType, // Type of the mouse event
}

// A thread-safe structure to store captured input events.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum MouseEventType {
    LeftDown,
    LeftUp,
    Move,
}

/// Shared list of clicked events, safely accessible across threads
pub type ClickLog = Arc<Mutex<Vec<ClickEvent>>>;

static ACTIVE_CLICK_LOG: OnceLock<Mutex<Option<ClickLog>>> = OnceLock::new();
static MOUSE_HOOK_STARTED: OnceLock<()> = OnceLock::new();
static LAST_MOVE: Mutex<(u64, i32, i32)> = Mutex::new((0, 0, 0));

/// Create a new empty click log
pub fn new_click_log() -> ClickLog {
    Arc::new(Mutex::new(Vec::new()))
}

fn active_click_log() -> &'static Mutex<Option<ClickLog>> {
    ACTIVE_CLICK_LOG.get_or_init(|| Mutex::new(None))
}

pub fn set_active_click_log(log: Option<ClickLog>) {
    *active_click_log().lock().unwrap() = log;
}

/// Get current timestamp in milliseconds since UNIX epoch
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Manually push a fake click — used for testing before real hook is built
pub fn push_test_click(log: &ClickLog, x: i32, y: i32) {
    let event = ClickEvent {
        timestamp_ms: now_ms(),
        x,
        y,
        event_type: MouseEventType::LeftDown,
    };

    log.lock().unwrap().push(event);
    println!("Click logged at ({}, {})", x, y);
}

pub fn start_mouse_hook_thread() {
    MOUSE_HOOK_STARTED.get_or_init(|| {
        thread::spawn(|| {
            #[cfg(target_os = "windows")]
            unsafe {
                run_mouse_hook_loop();
            }
        });
    });
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn mouse_hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    const HC_ACTION: i32 = 0;

    if code == HC_ACTION {
        let msg_type = w_param.0 as u32;
        let event_type = match msg_type {
            WM_LBUTTONDOWN => Some(MouseEventType::LeftDown),
            WM_LBUTTONUP => Some(MouseEventType::LeftUp),
            WM_MOUSEMOVE => Some(MouseEventType::Move),
            _ => None,
        };

        if let Some(event_type) = event_type {
            let info = *(l_param.0 as *const MSLLHOOKSTRUCT);
            let current_time = now_ms();

            let should_log = match event_type {
                MouseEventType::Move => {
                    let mut last = LAST_MOVE.lock().unwrap();
                    let (last_time, last_x, last_y) = *last;
                    let time_diff = current_time - last_time;
                    let coord_changed = info.pt.x != last_x || info.pt.y != last_y;
                    if time_diff >= 16 && coord_changed {
                        *last = (current_time, info.pt.x, info.pt.y);
                        true
                    } else {
                        false
                    }
                }
                _ => true,
            };

            if should_log {
                if let Some(log) = active_click_log().lock().unwrap().as_ref() {
                    log.lock().unwrap().push(ClickEvent {
                        timestamp_ms: current_time,
                        x: info.pt.x,
                        y: info.pt.y,
                        event_type,
                    });
                }
            }
        }
    }

    CallNextHookEx(HHOOK::default(), code, w_param, l_param)
}

#[cfg(target_os = "windows")]
unsafe fn run_mouse_hook_loop() {
    let hook = match SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), HINSTANCE::default(), 0)
    {
        Ok(hook) => hook,
        Err(error) => {
            log::error!("Failed to install low-level mouse hook: {error}");
            return;
        }
    };

    let mut message = MSG::default();
    while GetMessageW(&mut message, HWND(std::ptr::null_mut()), 0, 0).into() {
        let _ = TranslateMessage(&message);
        DispatchMessageW(&message);
    }

    let _ = UnhookWindowsHookEx(hook);
}
