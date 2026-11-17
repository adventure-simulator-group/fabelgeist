// Move the shared HTML conversation into the existing document rail. The
// canvas, runtime and scene keep their identity while this panel is open.
export function installLiveDialoguePanel() {
  let mounted;
  const restore = () => {
    if (!mounted) return;
    const { chat, chatWasHidden, lifecycle, placeholder, root, created, tab, pane, previous, hiddenTabs } = mounted;
    lifecycle.abort();
    chat.hidden = chatWasHidden;
    if (placeholder.isConnected) placeholder.replaceWith(chat);
    tab.remove(); pane.remove();
    root.removeAttribute("data-live-dialogue-panel");
    for (const entry of hiddenTabs) entry.tab.hidden = entry.hidden;
    if (created) root.remove();
    else if (root.isConnected) previous?.click();
    mounted = undefined;
  };
  const mount = () => {
    const context = window.tacticalDialogue?.current;
    if (!context) { restore(); return; }
    if (mounted?.root.isConnected) return;
    // A refreshed sidebar may have removed the rail, but the dock outside it
    // still owns the same chat node and its event handlers.
    restore();
    const chat = document.querySelector("[data-dialogue-chat]") || window.strategicConversationChat?.();
    const sidebar = document.querySelector(".left-sidebar");
    if (!chat || !sidebar) return;
    if (!chat.isConnected) {
      const dock = document.querySelector("[data-chat-dock]");
      if (!dock) return;
      chat.hidden = true;
      dock.append(chat);
      document.dispatchEvent(new Event("live-dialogue-chat-mounted"));
    }
    const chatWasHidden = chat.hidden;
    chat.hidden = false;
    const lifecycle = new AbortController();
    window.strategicChat?.mountFilters(chat, lifecycle.signal);
    let root = sidebar.querySelector("[data-side-panel='left']");
    const created = !root;
    if (!root) {
      root = document.createElement("div");
      root.className = "side-panel";
      root.dataset.sidePanel = "left";
      const nav = document.createElement("nav");
      nav.className = "side-panel-tabs";
      nav.setAttribute("role", "tablist");
      nav.setAttribute("aria-label", "Left panel");
      root.append(nav); sidebar.append(root);
    }
    const previous = root.querySelector('[data-panel-tab][aria-selected="true"]');
    const hiddenTabs = [...root.querySelectorAll("[data-panel-tab]")].map(tab => ({ tab, hidden: tab.hidden }));
    for (const { tab } of hiddenTabs) tab.hidden = true;
    root.dataset.liveDialoguePanel = "";
    const tab = document.createElement("button");
    tab.type = "button"; tab.className = "side-panel-tab";
    tab.id = "live-dialogue-tab";
    tab.dataset.panelTab = "dialogue";
    tab.setAttribute("role", "tab");
    tab.setAttribute("aria-controls", "live-dialogue-panel");
    tab.textContent = "Conversation";
    const pane = document.createElement("section");
    pane.className = "side-panel-content";
    pane.id = "live-dialogue-panel"; pane.dataset.panelPane = "dialogue";
    pane.setAttribute("role", "tabpanel");
    pane.setAttribute("aria-labelledby", tab.id);
    const close = document.createElement("button");
    close.type = "button"; close.className = "btn btn-small";
    close.dataset.liveDialogueClose = "";
    close.textContent = "End conversation";
    close.addEventListener("click", () => window.tacticalDialogue.close());
    const placeholder = document.createComment("conversation dock");
    chat.replaceWith(placeholder);
    pane.append(close, chat);
    root.querySelector("[role='tablist']").append(tab);
    root.append(pane);
    mounted = { chat, chatWasHidden, lifecycle, placeholder, root, created, tab, pane, previous, hiddenTabs };
    tab.click();
  };
  document.addEventListener("live-dialogue-changed", mount);
  document.addEventListener("strategic-page-unmounting", restore);
}
