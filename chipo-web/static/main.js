import init, {
  compile,
  new_emulator,
  cycle_emulator,
  get_display_buffer_emulator,
  decrement_registers_emulator,
} from "../pkg/chipo_web.js";

function clearScreen() {
  ctx.fillStyle = "rgb(50, 50, 50)";
  ctx.fillRect(0, 0, 64 * SCALE, 32 * SCALE);
}

const SCALE = 5;
const N_PIXELS = 64 * 32;
function Emulator(code) {
  this.emu = new_emulator(code);
  this._isPlaying = true;

  this.cycle = function () {
    const res = cycle_emulator(this.emu);
    if (res !== 0) {
      this._isPlaying = false;
    }
  };

  this.isPlaying = () => this._isPlaying;

  this.decrementRegisters = function () {
    decrement_registers_emulator(this.emu);
  };

  this.display = function () {
    const pixels = new Uint8Array(N_PIXELS);
    get_display_buffer_emulator(this.emu, pixels);

    ctx.fillStyle = "rgb(50, 50, 50)";
    ctx.fillRect(0, 0, 64 * SCALE, 32 * SCALE);

    ctx.fillStyle = "rgb(0, 255, 100)";
    for (let x = 0; x < 64; x++) {
      for (let y = 0; y < 32; y++) {
        if (pixels[y * 64 + x] !== 0) {
          ctx.fillRect(x * SCALE, y * SCALE, SCALE, SCALE);
        }
      }
    }
  };
}

function initEvent(event, cb) {
  const clickableSpan = document.getElementById(event);

  clickableSpan.addEventListener("click", () => {
    cb(clickableSpan);
  });
}

let autoSave = true;
initEvent("toggle-auto-save", (span) => {
  autoSave = !autoSave;
  span.innerText = autoSave ? "[x]" : "[  ]";
});

let autoRun = true;
initEvent("toggle-auto-run", (span) => {
  autoRun = !autoRun;
  span.innerText = autoRun ? "[x]" : "[  ]";
});

CodeMirror.defineSimpleMode("asm", {
  start: [
    { regex: /\.[a-zA-Z]+/, token: "keyword" },
    { regex: /[a-z0-9A-Z_]+:/, token: "variable" },
    { regex: /\b(i|\[i\]|f|k)\b/, token: "number" },
    { regex: /\bv[0-9A-Fa-f]\b/, token: "number" },
    { regex: /0x[0-9A-Fa-f]+/, token: "number" },
    { regex: /\d+/, token: "number" },
    { regex: /;.*/, token: "comment" },
  ],
  meta: {
    lineComment: ";",
  },
});

let codeValue = localStorage.getItem("saved-code") || "";
let running = false;

const canvas = document.getElementById("canvas");
canvas.width = 64 * SCALE;
canvas.height = 32 * SCALE;
const errorSection = document.querySelector(".error");
const hideError = () => {
  errorSection.style.display = "none";
};
const showError = (err) => {
  errorSection.style.display = "block";
  errorSection.innerText = `error: ${err}`;
};

const ctx = canvas.getContext("2d");
async function run() {
  if (codeValue.length == 0) {
    codeValue = await fetch(
      "https://raw.githubusercontent.com/Pangoraw/chipo/main/roms/chipo.s"
    ).then((res) => res.text());
  }

  const textArea = document.getElementById("code-mirror");
  const codeMirror = CodeMirror(textArea, {
    value: codeValue,
    lineNumbers: true,
    mode: "asm",
    theme: "material-darker",
  });
  initEvent("clear", () => {
    codeMirror.setValue("");
  });
  const save = () => {
    const codeValue = codeMirror.getValue();
    localStorage.setItem("saved-code", codeValue);
  };
  initEvent("save", save);
  const stop = () => {
    running = false;
    clearScreen();
  };
  initEvent("stop", stop);

  const roms = document.querySelector(".lib").children[1].children[0].children;
  for (let rom of roms) {
    rom.addEventListener("click", async () => {
      const url = rom.dataset.src;
      const newText = await fetch(url).then((res) => res.text());
      codeMirror.setValue(newText);
    });
  }

  await init();

  const startEmu = () => {
    const code_buffer = new Uint8Array(100);
    const text = codeMirror.getValue();
    try {
      compile(text, code_buffer);
    } catch (err) {
      showError(err);
      return;
    }
    hideError();

    const emu = new Emulator(code_buffer);
    running = true;

    function loop() {
      if (!emu.isPlaying() || !running) {
        return;
      }

      for (let i = 0; i < 10; i++) {
        emu.cycle();
      }
      emu.decrementRegisters();
      emu.display();

      requestAnimationFrame(loop);
    }

    loop();
  };

  const runAgain = async () => {
    stop();
    await new Promise((res) => setTimeout(res));
    startEmu();
  };

  initEvent("run", runAgain);
  codeMirror.on("change", () => {
    if (autoSave) {
      save();
    }
    if (autoRun) {
      runAgain();
    }
  });
  startEmu();
}

run();
