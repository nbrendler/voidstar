import { Game } from "wasm-webgl-demo";

const game = Game.new();
const TIME_PER_FRAME = 1000 / 60;

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
