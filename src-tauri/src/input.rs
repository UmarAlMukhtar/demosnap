#![allow(dead_code)]

use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// This module defines the structure for input events captured during a recording session.
#[derive(Debug, Clone, Serialize)]
pub struct ClickEvent {
    pub timestamp_ms: u64, // Timestamp in milliseconds since UNIX epoch
    pub x: i32, // X coordinate of the event
    pub y: i32, // Y coordinate of the event
    pub event_type: MouseEventType, // Type of the mouse event
}

// A thread-safe structure to store captured input events.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MouseEventType {
    LeftDown,
    LeftUp,
    Move,
}

/// Shared list of clicked events, safely accessible across threads
pub type ClickLog = Arc<Mutex<Vec<ClickEvent>>>;

/// Create a new empty click log
pub fn new_click_log() -> ClickLog {
    Arc::new(Mutex::new(Vec::new()))
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

