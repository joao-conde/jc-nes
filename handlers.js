import { Nes, Button } from "./jc_nes.js";

import { getROM } from "./roms.js";
import { KEY_MAPPER } from "./keys.js";
import { updateCanvas } from "./canvas.js";

let nes = null;

export const play = async () => {
    const rom = await getROM();
    nes = new Nes();
    nes.load_rom(rom);
    nes.reset();
};

export const clock = () => {
    if (nes) nes.clock();
};

export const render = () => {
    if (!nes) return;

    const frame = nes.get_frame();
    if (!frame) return;

    updateCanvas(frame);
};

export const onKeyDown = (event) => {
    const key = KEY_MAPPER[event.key];
    if (key === undefined) return;
    document.querySelector(`#key-${event.key}`).style.opacity = "1";
};

export const onKeyUp = (event) => {
    const key = KEY_MAPPER[event.key];
    if (key === undefined) return;
    document.querySelector(`#key-${event.key}`).style.opacity = "0.3";
};
