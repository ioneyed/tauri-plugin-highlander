#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri_plugin_highlander::*;

fn main() {
  tauri::Builder::default()
    .plugin(HighlanderBuilder::default().build())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
/*
tauri::Builder::default()
    .plugin(Highlander::default())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
*/

/*
tauri::Builder::default()
    .plugin(HighlanderBuilder::new()
      .event("")
      .listen("")
      .broadcaster(<function>)
    )
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
*/