const canvas = document.createElement("canvas");
canvas.width = 256;
canvas.height = 240;
const ctx = canvas.getContext("2d");

const scaledCanvas = document.querySelector("canvas#scaled");
const scaledCtx = scaledCanvas.getContext("2d");
scaledCtx.scale(scaledCanvas.width / canvas.width, scaledCanvas.height / canvas.height);
scaledCtx.imageSmoothingEnabled = false;

const image = ctx.createImageData(canvas.width, canvas.height);
const videoBuff = new DataView(image.data.buffer);

export function updateCanvas(pixels) {
    for (let i = 0; i < pixels.length; i++) {
        videoBuff.setUint8(i, pixels[i]);
    }
    ctx.putImageData(image, 0, 0);
    scaledCtx.drawImage(canvas, 0, 0);
}
