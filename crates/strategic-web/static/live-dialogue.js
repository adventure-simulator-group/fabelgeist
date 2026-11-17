// The native host grants the session. HTTP only reads its persisted transcript.
export function installLiveDialogue(command) {
  let current;
  let view;
  const pending = new Map();
  const publish = () => document.dispatchEvent(new CustomEvent("live-dialogue-changed", {
    detail: current ? { ...current, view } : null,
  }));
  const clear = () => {
    current = undefined;
    view = undefined;
    for (const action of pending.values()) action.reject(new Error("Conversation ended"));
    pending.clear();
    publish();
  };
  const read = async (context) => {
    const response = await window.strategicFetch(`/api/dialogue/${encodeURIComponent(context.session_id)}`, {
      headers: { Accept: "application/json" },
    });
    if (!response.ok) throw new Error(`Conversation unavailable (${response.status})`);
    const next = await response.json();
    if (current !== context || next.session_id !== context.session_id) throw new Error("Conversation changed");
    // Concurrent receipts may finish their reads in either order.
    if (!view || next.revision >= view.revision) {
      view = next;
      publish();
    }
    return view;
  };
  const request = (path, payload) => {
    const context = current;
    if (!context || payload.session_id !== context.session_id) return Promise.reject(new Error("Conversation ended"));
    const { session_id, ...fields } = payload;
    const kind = ({ "/api/dialogue/topic": "Topic", "/api/dialogue/answer": "Answer" })[path];
    if (!kind) return Promise.reject(new Error("This action is unavailable during live conversation"));
    const existing = pending.get(fields.action_id);
    if (existing) return existing.promise;
    let resolve, reject;
    const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
    pending.set(fields.action_id, { promise, resolve, reject, context });
    command({ type: "dialogue", request: { session_id, action: { [kind]: fields } } });
    return promise;
  };
  const close = () => {
    if (!current) return;
    command({ type: "dialogue", request: { session_id: current.session_id, action: "Close" } });
    clear();
    document.querySelector("#game-canvas")?.focus({ preventScroll: true });
  };
  window.tacticalDialogue = {
    get current() { return current; },
    get view() { return view; },
    request, close,
  };
  document.addEventListener("tactical-dialogue", async ({ detail }) => {
    if (detail.type === "opened") {
      if (!document.body.hasAttribute("data-tactical-active")) return;
      clear();
      const context = { session_id: detail.session_id, resident_id: detail.resident_id };
      current = context;
      publish();
      try { await read(context); }
      catch (error) { if (current === context) window.reportStrategicError(error, "read live conversation"); }
    } else if (detail.type === "closed" && current?.session_id === detail.session_id) {
      clear();
    } else if (detail.type === "action" && current?.session_id === detail.session_id) {
      const action = pending.get(detail.action_id);
      if (!action || action.context !== current) return;
      pending.delete(detail.action_id);
      if (Object.hasOwn(detail.result, "Err")) action.reject(new Error(detail.result.Err));
      else {
        try { action.resolve(await read(action.context)); }
        catch (error) { action.reject(error); }
      }
    } else if (detail.type === "unavailable") {
      window.reportStrategicError(new Error(detail.message), "start live conversation");
    }
  });
  document.addEventListener("strategic-tactical-ended", clear);
  // Mount listeners rebuild their renderer before this snapshot is replayed.
  for (const event of ["strategic-page-mounted", "strategic-live-regions-refreshed"]) {
    document.addEventListener(event, () => queueMicrotask(publish));
  }
  window.addEventListener("keydown", event => {
    if (event.key !== "Escape" || !current) return;
    if (event.target.closest?.("dialog, [role='dialog'], .character-action-overlay")) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    close();
  }, true);
}
