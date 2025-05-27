import { default as wasm } from "./jc_nes.js";

import { listROMs } from "./roms.js";
import { listKeys } from "./keys.js";
import { play, clock, render, onKeyDown, onKeyUp } from "./handlers.js";

const FPS = 10_000;
const CLOCK_HZ = 10_000;

(async () => {
    // init wasm module
    await wasm();

    // set clock, video, audio and keyboard handlers
    window.setInterval(clock, 1000 / CLOCK_HZ);
    window.setInterval(render, 1000 / FPS);
    window.onkeydown = onKeyDown;
    window.onkeyup = onKeyUp;

    // play ROM on change
    document.querySelector('#roms').onchange = e => play(e.target.value);

    listROMs();
    listKeys();
})();
