// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  // Enforce 1:1 display scale immunity and disable zoom gestures/hotkeys in WebView2
  unsafe {
    std::env::set_var(
      "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
      "--force-device-scale-factor=1 --disable-pinch --disable-features=msWebview2ZoomHotkeys",
    );
  }
  app_lib::run();
}
