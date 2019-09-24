import {NesEmulator} from "wasm-nes";

const canvas = document.getElementById("nes-canvas");
const emulator = NesEmulator.new();
const width = emulator.width();
const height = emulator.height();

const CELL_SIZE = 3;
const GRID_COLOR = "#CCCCCC";

// Set the size of the canvas
canvas.width = CELL_SIZE * width;
canvas.height = CELL_SIZE * height;
const ctx = canvas.getContext('2d');



emulator.tick();
emulator.log_cpu();

emulator.tick();
emulator.log_cpu();

emulator.tick();
emulator.log_cpu();
emulator.tick();
emulator.log_cpu();
emulator.tick();
emulator.log_cpu();
emulator.tick();
emulator.log_cpu();
emulator.tick();
emulator.log_cpu();

const drawGrid = () => {
  ctx.beginPath();
 ctx.lineWidth = 5;
  ctx.strokeStyle = GRID_COLOR;

  // Horizontal lines.
  ctx.moveTo(0, 0);
  ctx.lineTo(0, CELL_SIZE*height);
  ctx.moveTo(CELL_SIZE*width, 0);
  ctx.lineTo(CELL_SIZE*width, CELL_SIZE*height);

  // Vertical lines.
  ctx.moveTo(0, 0);
  ctx.lineTo(CELL_SIZE*width, 0);
  ctx.moveTo(0, CELL_SIZE*height);
  ctx.lineTo(CELL_SIZE*width, CELL_SIZE*height);

  ctx.stroke();
};

const drawPixels = () => {

    

}

drawGrid();
