// The scene keeps simulating while HTML owns keyboard or pointer interaction.
(() => {
  const interfaceSelector = "[data-side-panel], [data-chat-dock], [role='dialog'], .character-action-overlay, input, textarea, select, [contenteditable='true']";
  let pointerTarget;
  const isInterface = target => target instanceof Element && target.isConnected
    && !target.closest("[hidden]") && Boolean(target.closest(interfaceSelector));
  document.addEventListener("pointermove", event => { pointerTarget = event.target; }, true);
  document.addEventListener("pointerdown", event => {
    pointerTarget = event.target;
    // Clicking the world returns keyboard ownership even after editing a form.
    if (event.target instanceof HTMLCanvasElement && event.target.id === "game-canvas") {
      event.target.focus({ preventScroll: true });
    }
  }, true);
  document.addEventListener("pointerleave", () => { pointerTarget = undefined; });
  window.addEventListener("blur", () => { pointerTarget = undefined; });
  window.tacticalUiOwnsInput = () => document.hidden
    || !document.hasFocus() || isInterface(document.activeElement) || isInterface(pointerTarget);
})();
