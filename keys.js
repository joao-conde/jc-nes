export const KEY_MAPPER = {
    "a": 0x80,
    "b": 0x40,
    "z": 0x10,
    "x": 0x20,
};

export const listKeys = () => {
    const keyboard = document.querySelector('#keyboard');
    Object.keys(KEY_MAPPER).forEach(key => {
        const div = document.createElement("div");
        div.id = `key-${key}`;
        div.innerHTML = key.toUpperCase();
        div.className = "key";
        keyboard.appendChild(div);
    })
}
