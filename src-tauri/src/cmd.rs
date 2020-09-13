use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  // your custom commands
  // multiple arguments are allowed
  // note that rename_all = "camelCase": you need to use "myCustomCommand" on JS
  StartSynth {
    argument: String,
    callback: String,
    error: String,
  }, //, payload: Payload },
}

#[derive(Deserialize)]
pub struct Payload {}
