// Generates icons/icon.png (512) and icons/icon.ico (256, PNG-in-ICO)
// with no external dependencies — only Node's built-in zlib.
const fs = require("fs");
const path = require("path");
const zlib = require("zlib");

const OUT = path.join(__dirname, "..", "icons");
fs.mkdirSync(OUT, { recursive: true });

function crc32(buf) {
  let c = ~0;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return (~c) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const typeBuf = Buffer.from(type, "ascii");
  const crcBuf = Buffer.alloc(4);
  crcBuf.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])), 0);
  return Buffer.concat([len, typeBuf, data, crcBuf]);
}

// Render a round clock-ish glyph into an RGBA buffer.
function renderRGBA(size) {
  const buf = Buffer.alloc(size * size * 4);
  const cx = size / 2, cy = size / 2;
  const R = size * 0.46;
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const dx = x - cx, dy = y - cy;
      const d = Math.hypot(dx, dy);
      let r = 8, g = 12, b = 22, a = 0; // transparent backdrop
      if (d <= R) {
        // radial dark-blue glass gradient
        const t = d / R;
        r = Math.round(18 + 26 * (1 - t));
        g = Math.round(26 + 40 * (1 - t));
        b = Math.round(46 + 70 * (1 - t));
        a = 255;
        // bright rim
        if (d > R - size * 0.018) {
          r = 150; g = 180; b = 240; a = 255;
        }
      }
      // hands (12 o'clock hour + 3 o'clock minute-ish), amber
      const ang = Math.atan2(dy, dx);
      const handHour = Math.abs(angDiff(ang, -Math.PI / 2)) < 0.10 && d < R * 0.55;
      const handMin = Math.abs(angDiff(ang, -Math.PI / 6)) < 0.07 && d < R * 0.78;
      if ((handHour || handMin) && d < R) {
        r = 255; g = 198; b = 110; a = 255;
      }
      // center cap
      if (d < size * 0.03) { r = 245; g = 248; b = 252; a = 255; }
      const o = (y * size + x) * 4;
      buf[o] = r; buf[o + 1] = g; buf[o + 2] = b; buf[o + 3] = a;
    }
  }
  return buf;
}

function angDiff(a, b) {
  let d = a - b;
  while (d > Math.PI) d -= 2 * Math.PI;
  while (d < -Math.PI) d += 2 * Math.PI;
  return d;
}

function encodePNG(size, rgba) {
  // add filter byte (0) per scanline
  const stride = size * 4;
  const raw = Buffer.alloc((stride + 1) * size);
  for (let y = 0; y < size; y++) {
    raw[y * (stride + 1)] = 0;
    rgba.copy(raw, y * (stride + 1) + 1, y * stride, y * stride + stride);
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0);
  ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8;   // bit depth
  ihdr[9] = 6;   // color type RGBA
  ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0;
  const sig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  return Buffer.concat([
    sig,
    chunk("IHDR", ihdr),
    chunk("IDAT", zlib.deflateSync(raw, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

function pngToIco(pngBuf, size) {
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2); // type icon
  header.writeUInt16LE(1, 4); // count
  const entry = Buffer.alloc(16);
  entry[0] = size >= 256 ? 0 : size; // width (0 => 256)
  entry[1] = size >= 256 ? 0 : size; // height
  entry[2] = 0; // palette
  entry[3] = 0;
  entry.writeUInt16LE(1, 4);  // planes
  entry.writeUInt16LE(32, 6); // bpp
  entry.writeUInt32LE(pngBuf.length, 8);
  entry.writeUInt32LE(6 + 16, 12); // offset
  return Buffer.concat([header, entry, pngBuf]);
}

const png512 = encodePNG(512, renderRGBA(512));
fs.writeFileSync(path.join(OUT, "icon.png"), png512);

const png256 = encodePNG(256, renderRGBA(256));
fs.writeFileSync(path.join(OUT, "icon.ico"), pngToIco(png256, 256));

// A couple of extra sizes Tauri likes to reference.
for (const s of [32, 128]) {
  fs.writeFileSync(path.join(OUT, `${s}x${s}.png`), encodePNG(s, renderRGBA(s)));
}
fs.writeFileSync(path.join(OUT, "128x128@2x.png"), encodePNG(256, renderRGBA(256)));

console.log("icons written to", OUT);
