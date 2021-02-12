import init, {
  compile,
  new_emulator,
  cycle_emulator,
  get_display_buffer_emulator,
  decrement_registers_emulator,
	should_buzz,
	set_key_down_emulator,
	set_key_up_emulator,
	reverse_parse,
} from "../pkg/chipo_web.js";

function clearScreen() {
  ctx.fillStyle = "rgb(50, 50, 50)";
  ctx.fillRect(0, 0, 64 * SCALE, 32 * SCALE);
}

const MAX_SIZE = 2048;
const SCALE = 5;
const N_PIXELS = 64 * 32;
let audioCtx = new (window.AudioContext || window.webkitAudioContext)();
function Emulator(code) {
	// this.emu is a reference to Rust Emulator struct. 
  this.emu = new_emulator(code);
	this._newOscillator = () => {
		const oscillator = audioCtx.createOscillator();
		oscillator.type = "square";
		oscillator.frequency.value = 400;
		oscillator.connect(audioCtx.destination);
		return oscillator;
	}
	this._oscillator = this._newOscillator();

  this._isPlaying = true;

  this.cycle = function () {
    const res = cycle_emulator(this.emu);
    if (res !== 0) {
      this._isPlaying = false;
    }
  };

  this.isPlaying = () => this._isPlaying;
	this.doBuzz = () => {
		if (!this._audioRunning && should_buzz(this.emu)) {
			this._oscillator.start();
			this._audioRunning = true;
		} else if (this._audioRunning && !should_buzz(this.emu)) {
			this._oscillator.stop();
			this._audioRunning = false;
			this._oscillator = this._newOscillator();
		}
	}

	this.setKeyUp = function(key) {
		set_key_up_emulator(this.emu, key);
	}

	this.setKeyDown = function(key) {
		set_key_down_emulator(this.emu, key);
	}

  this.decrementRegisters = function () {
    decrement_registers_emulator(this.emu);
  };

	this.destroy = () => {
		if (this._audioRunning)
			this._oscillator.stop();
	}

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
const fileUploader = document.getElementById("c8file");
window.fileUploader = fileUploader;

const ctx = canvas.getContext("2d");
let emu = null;
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
		audioCtx.close();
		audioCtx = new(window.AudioContext || window.webkitAudioContext)();
    clearScreen();
  };
  initEvent("stop", stop);

	document.addEventListener("keydown", function(event) {
		const key = event.code.toLowerCase();

		if (emu) {
			emu.setKeyDown(key);
		}
	});
	document.addEventListener("keyup", function(event) {
		const key = event.code.toLowerCase();

		if (emu) {
			emu.setKeyUp(key);
		}
	});

  const roms = document.querySelector(".roms").children;
  for (let rom of roms) {
    rom.addEventListener("click", async () => {
      const url = rom.dataset.src;
      const newText = await fetch(url).then((res) => res.text());
      codeMirror.setValue(newText);
    });
  }

  await init();

  const startEmu = () => {
    let code_buffer = new Uint8Array(MAX_SIZE);
    const text = codeMirror.getValue();
    try {
      let length = compile(text, code_buffer);
			code_buffer = code_buffer.slice(0, length);
    } catch (err) {
      showError(err);
      return;
    }
    hideError();

		if (emu !== null) {
			emu.destroy();
		}
    emu = new Emulator(code_buffer);
    running = true;
		audioCtx = new (window.AudioContext || window.webkitAudioContext)();

    function loop() {
      if (!emu.isPlaying() || !running) {
        return;
      }

      for (let i = 0; i < 10; i++) {
        emu.cycle();
      }
      emu.decrementRegisters();
      emu.display();
			emu.doBuzz();

      requestAnimationFrame(loop);
    }

    loop();
  };

  const runAgain = async () => {
    stop();
    await new Promise((res) => setTimeout(res));
    startEmu();
  };

	document.querySelector(".download").addEventListener("click", async () => {
		let data = new Uint8Array(MAX_SIZE);
		const length = compile(codeMirror.getValue(), data);
		data = data.slice(0, length);
		const file = new Blob([data]);
		const url = URL.createObjectURL(file),
					a = document.createElement("a");
		a.href = url;
		a.download = "program.c8";

		document.body.appendChild(a);
		a.click();
		setTimeout(function() {
			document.body.removeChild(a);
			window.URL.revokeObjectURL(url);
		});
	});
	document.querySelector(".load").addEventListener("click", async () => {
		if (!fileUploader.files.length) showError("no file uploaded")

		const fileContent = await fileUploader.files[0].arrayBuffer();
		const result = reverse_parse(new Uint8Array(fileContent));
		codeMirror.setValue(result);
	});
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
