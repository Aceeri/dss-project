#![recursion_limit = "256"]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate mimalloc;
extern crate reqwest;
extern crate serde_json;
extern crate tokio;
extern crate uuid;
extern crate winit;
#[macro_use]
extern crate anyhow;
extern crate image;

pub mod app;
pub mod grabber;
pub mod home;
pub mod menu;
pub mod renderer;
pub mod util;

pub fn hide_console_window() {
    use std::ptr;
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::winuser::{ShowWindow, SW_HIDE};

    let window = unsafe {GetConsoleWindow()};
    // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
    if window != ptr::null_mut() {
        unsafe {
            ShowWindow(window, SW_HIDE);
        }
    }
}