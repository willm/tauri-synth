document.onload = window.onload = () => {
  const element = document.getElementById("main");
  element.innerHTML = "Hi from JS";

  const promisified = window.__TAURI__.tauri.promisified;
  const button = document.getElementById("rust");
  button.addEventListener("click", async () => {
    const result = await promisified({
      cmd: "startSynth",
      argument: "Hi Rust",
    });
    element.innerHTML = result;
  });
};
