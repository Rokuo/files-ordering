import { invoke } from "@tauri-apps/api/core";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;

let listfilesMsgEl: HTMLElement | null;

async function greet() {
  console.log('oui');
  if (greetMsgEl && greetInputEl) {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsgEl.textContent = await invoke("greet", {
      name: greetInputEl.value,
    });
  }
}

async function listFiles() {
  if (listfilesMsgEl) {
    alert('coucou');
    listfilesMsgEl.textContent = await invoke("list_files", {
      name: "osef"
    });
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  listfilesMsgEl = document.querySelector("#listfiles-msg");
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
  document.querySelector("#listfiles-form")?.addEventListener("submit", (e) => {
    alert('oui');
    e.preventDefault();
    listFiles();
  })
});
