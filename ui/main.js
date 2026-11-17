// Settings window. Uses the global Tauri API (app.withGlobalTauri = true).
const { invoke } = window.__TAURI__.core;

const $ = (id) => document.getElementById(id);
const checkboxes = [
  "readSelectionViaUiAutomation",
  "reselectAfterReplace",
  "restoreClipboard",
  "launchAtLogin",
  "checkForUpdates",
];

let settings = null;

function setStatus(text, kind = "") {
  const s = $("status");
  s.textContent = text;
  s.className = kind;
}

// ---------- form <-> settings ----------

function fill(s) {
  $("hotkeyToggle").value = s.hotkeyToggle;
  $("hotkeyAlongside").value = s.hotkeyAlongside;

  const preset = [...$("templatePreset").options].find((o) => o.value === s.alongsideTemplate);
  $("templatePreset").value = preset ? preset.value : "custom";
  $("template").value = s.alongsideTemplate;
  $("template").classList.toggle("hidden", !!preset);

  document.querySelector(`input[name=scheme][value=${s.transliteration.scheme}]`).checked = true;
  $("nukta").checked = s.transliteration.nukta;
  $("convertDigits").checked = s.transliteration.convertDigits;

  for (const id of checkboxes) $(id).checked = s[id];
  $("outputMethod").value = s.outputMethod;
}

function read() {
  const preset = $("templatePreset").value;
  return {
    ...settings,
    hotkeyToggle: $("hotkeyToggle").value,
    hotkeyAlongside: $("hotkeyAlongside").value,
    alongsideTemplate: preset === "custom" ? $("template").value || "{original} ({result})" : preset,
    transliteration: {
      scheme: document.querySelector("input[name=scheme]:checked").value,
      nukta: $("nukta").checked,
      convertDigits: $("convertDigits").checked,
    },
    ...Object.fromEntries(checkboxes.map((id) => [id, $(id).checked])),
    outputMethod: $("outputMethod").value,
  };
}

// ---------- live preview ----------

async function updatePreview() {
  const text = $("tryInput").value;
  const s = read();
  const [t, a] = await Promise.all([
    invoke("preview", { text, alongside: false, settings: s }),
    invoke("preview", { text, alongside: true, settings: s }),
  ]);
  $("tryToggle").textContent = t ?? "—";
  $("tryAlongside").textContent = a ?? "—";
}

// ---------- hotkey recorder ----------

const MODIFIER_CODES = /^(Control|Shift|Alt|Meta|OS)(Left|Right)?$/;

function keyName(code) {
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  return code; // F1…F24, Space, ArrowLeft, Numpad1, Backquote, …
}

function setupRecorder(input) {
  let previous = "";
  input.addEventListener("focus", () => {
    previous = input.value;
    input.value = "Press keys…";
    input.classList.add("recording");
  });
  input.addEventListener("blur", () => {
    if (input.value === "Press keys…") input.value = previous;
    input.classList.remove("recording");
  });
  input.addEventListener("keydown", (e) => {
    e.preventDefault();
    if (e.key === "Escape") {
      input.value = previous;
      input.blur();
      return;
    }
    if (MODIFIER_CODES.test(e.code)) return;
    const mods = [];
    if (e.ctrlKey) mods.push("Ctrl");
    if (e.shiftKey) mods.push("Shift");
    if (e.altKey) mods.push("Alt");
    if (e.metaKey) mods.push("Super");
    const isFunctionKey = /^F\d+$/.test(e.code);
    if (mods.length === 0 && !isFunctionKey) {
      setStatus("Use at least one of Ctrl, Alt or Win (or a function key).", "error");
      return;
    }
    if (mods.length === 1 && mods[0] === "Shift" && !isFunctionKey) {
      setStatus("Shift alone would block typing capital letters.", "error");
      return;
    }
    input.value = [...mods, keyName(e.code)].join("+");
    setStatus("Not saved yet.");
    input.blur();
  });
}

// ---------- init ----------

async function init() {
  settings = await invoke("get_settings");
  fill(settings);
  $("version").textContent = "v" + (await invoke("app_version"));

  setupRecorder($("hotkeyToggle"));
  setupRecorder($("hotkeyAlongside"));

  $("templatePreset").addEventListener("change", () => {
    const custom = $("templatePreset").value === "custom";
    $("template").classList.toggle("hidden", !custom);
    if (custom) $("template").focus();
  });

  document.querySelectorAll("input, select, textarea").forEach((el) => {
    el.addEventListener("input", () => {
      updatePreview();
      if (el.id !== "tryInput") setStatus("Not saved yet.");
    });
  });

  $("save").addEventListener("click", async () => {
    try {
      const s = read();
      await invoke("save_settings", { settings: s });
      settings = s;
      setStatus("Saved.", "ok");
    } catch (err) {
      setStatus(String(err), "error");
    }
  });

  $("reset").addEventListener("click", async () => {
    fill({ ...(await invoke("default_settings")) });
    updatePreview();
    setStatus("Defaults loaded – press Save to apply.");
  });

  $("checkNow").addEventListener("click", () => invoke("check_for_updates"));

  updatePreview();
}

init().catch((e) => setStatus(String(e), "error"));
