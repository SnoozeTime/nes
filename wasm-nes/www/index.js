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

var Key = {
  _pressed: {},

  LEFT: 37,
  UP: 38,
  RIGHT: 39,
  DOWN: 40,
  
  isDown: function(keyCode) {
    return this._pressed[keyCode];
  },
  
  onKeydown: function(event) {
    this._pressed[event.keyCode] = true;
  },
  
  onKeyup: function(event) {
    delete this._pressed[event.keyCode];
  }
};



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

  ctx.beginPath();

  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const color = emulator.get_pixel(row, col);


      ctx.fillStyle = `rgb(
        ${color.r()},
        ${color.g()},
        ${color.b()})`;

      ctx.fillRect(
        col * CELL_SIZE,
        row * CELL_SIZE,
        CELL_SIZE,
        CELL_SIZE
      );
    }
  }

  ctx.stroke();
};


const renderLoop = () => {

   // for (var i = 0; i < 29780; i++) {
   //     emulator.tick();
   // }

    //console.time("EMU");
    emulator.run_bunch_of_ticks();
    //console.timeEnd("EMU");

    //console.time("DISPLAY");
    //drawGrid();
    //drawPixels();
    //console.timeEnd("DISPLAY");
    console.log("hi");
  requestAnimationFrame(renderLoop);
};

drawGrid();
drawPixels();
requestAnimationFrame(renderLoop);
