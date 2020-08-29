import { Game } from "wasm-webgl-demo";

const game = Game.new();
const TIME_PER_FRAME = 1000 / 60;

const canvas = document.getElementById("game");

canvas.focus();
canvas.addEventListener("mousedown", (e) => {
  e.stopPropagation();
  game.log_mousedown_event(e);
});
canvas.addEventListener("mouseup", (e) => {
  e.stopPropagation();
  game.log_mouseup_event(e);
});
canvas.addEventListener("keydown", (e) => {
  e.stopPropagation();
  game.log_keydown_event(e);
});
canvas.addEventListener("keyup", (e) => {
  e.stopPropagation();
  game.log_keyup_event(e);
});

let lastCall = Date.now();

const renderLoop = () => {
  requestAnimationFrame(renderLoop);
  let elapsed = Date.now() - lastCall;
  if (elapsed >= TIME_PER_FRAME) {
    game.tick();
    lastCall = Date.now() - (elapsed % TIME_PER_FRAME);
  }
};

requestAnimationFrame(renderLoop);
