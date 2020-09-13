#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
  tauri::AppBuilder::new()
    .setup(|webview, _source| {
      let mut webview = webview.as_mut();
      tauri::event::listen(String::from("start-synth"), move |msg| {
        println!("got js-event with message '{:?}'", msg);

        let start = SystemTime::now();
        let since_the_epoch = start
          .duration_since(UNIX_EPOCH)
          .expect("Time went backwards");
        tauri::event::emit(
          &mut webview,
          String::from("message"),
          Some(format!("Current time is {:?}", since_the_epoch)),
        )
        .expect("failed to emit");
      });
    })
    .invoke_handler(|webview, arg| {
      use cmd::Cmd::*;
      match serde_json::from_str(arg) {
        Err(e) => Err(e.to_string()),
        Ok(command) => {
          match command {
            // definitions for your custom commands from Cmd here
            StartSynth {
              argument,
              callback,
              error,
            } => tauri::execute_promise(
              webview,
              move || {
                //  your command code
                println!("{}", argument);
                Ok("Response from rust")
              },
              callback,
              error,
            ),
          }
          Ok(())
        }
      }
    })
    .build()
    .run();
}
