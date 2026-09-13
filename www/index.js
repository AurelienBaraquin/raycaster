import init, { Game } from "./pkg/raycaster_web.js";

// event.code = position physique de la touche, independante du layout clavier
// actif (donc "KeyW" reste "KeyW" meme si l'utilisateur est en AZERTY et que
// cette touche affiche/tape un 'Z') — coherent avec le choix fait cote natif.
const keysDown = new Set();
window.addEventListener("keydown", (event) => keysDown.add(event.code));
window.addEventListener("keyup", (event) => keysDown.delete(event.code));

function readInput() {
  let forward = 0;
  let strafe = 0;
  let turn = 0;

  if (keysDown.has("KeyW")) forward += 1;
  if (keysDown.has("KeyS")) forward -= 1;
  if (keysDown.has("KeyD")) strafe += 1;
  if (keysDown.has("KeyA")) strafe -= 1;
  // sens inverse par rapport a l'intuition "droite = +1" : meme correction
  // que celle appliquee cote natif apres test manuel (rotation ressentie a
  // l'envers avec le mapping direct).
  if (keysDown.has("ArrowRight")) turn -= 1;
  if (keysDown.has("ArrowLeft")) turn += 1;

  return { forward, strafe, turn };
}

async function main() {
  await init();
  const game = new Game();

  let lastTime = performance.now();

  function frame(now) {
    const dt = (now - lastTime) / 1000;
    lastTime = now;

    const input = readInput();
    game.update(input.forward, input.strafe, input.turn, dt);
    game.render();

    requestAnimationFrame(frame);
  }

  requestAnimationFrame(frame);
}

main();
