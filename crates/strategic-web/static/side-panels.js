(() => {
  // Retain only the current subject on each side; visiting people cannot grow
  // a document-lifetime cache of their content or tab state.
  const selection = new Map();
  let focusedTab;
  const roots = () => [...document.querySelectorAll("[data-side-panel]")];
  const tabs = root => [...root.querySelectorAll("[data-panel-tab]")];
  const select = (root, tab, focus = false) => {
    if (!tab || tab.closest("[data-side-panel]") !== root) return;
    const id = tab.dataset.panelTab;
    tabs(root).forEach(entry => {
      const active = entry === tab;
      entry.setAttribute("aria-selected", String(active));
      entry.tabIndex = active ? 0 : -1;
    });
    root.querySelectorAll("[data-panel-pane]").forEach(pane => {
      pane.hidden = pane.dataset.panelPane !== id;
    });
    selection.set(root.dataset.sidePanel, { context: root.dataset.panelContext, id });
    if (focus) tab.focus({ preventScroll: true });
    const hasInventory = [...root.querySelectorAll("[data-inventory-browser]")]
      .some(browser => !browser.closest("[hidden]"));
    if (!hasInventory) root.closest(".main-grid")?.style.removeProperty(`--inventory-${root.dataset.sidePanel}-width`);
    window.strategicInventoryBrowser?.refresh?.(document);
    document.dispatchEvent(new CustomEvent("side-panel-selected", {
      detail: { side: root.dataset.sidePanel, context: root.dataset.panelContext, kind: id },
    }));
  };
  const mount = () => roots().forEach(root => {
    const previous = selection.get(root.dataset.sidePanel);
    const entries = tabs(root);
    const remembered = previous?.context === root.dataset.panelContext
      && entries.find(tab => tab.dataset.panelTab === previous.id);
    const tab = remembered || entries.find(tab => tab.getAttribute("aria-selected") === "true") || entries[0];
    const restoreFocus = focusedTab?.side === root.dataset.sidePanel
      && focusedTab.context === root.dataset.panelContext && focusedTab.id === tab?.dataset.panelTab;
    select(root, tab, restoreFocus);
  });
  document.addEventListener("focusin", event => {
    const tab = event.target.closest?.("[data-panel-tab]");
    const root = tab?.closest("[data-side-panel]");
    focusedTab = root ? { side: root.dataset.sidePanel, context: root.dataset.panelContext, id: tab.dataset.panelTab } : undefined;
  });
  document.addEventListener("click", event => {
    const tab = event.target.closest?.("[data-panel-tab]");
    if (tab) select(tab.closest("[data-side-panel]"), tab);
  });
  document.addEventListener("keydown", event => {
    const tab = event.target.closest?.("[data-panel-tab]");
    if (!tab) return;
    const root = tab.closest("[data-side-panel]");
    const entries = tabs(root);
    const index = entries.indexOf(tab);
    let next;
    switch (event.key) {
      case "ArrowRight": next = entries[(index + 1) % entries.length]; break;
      case "ArrowLeft": next = entries[(index + entries.length - 1) % entries.length]; break;
      case "Home": next = entries[0]; break;
      case "End": next = entries.at(-1); break;
      default: return;
    }
    event.preventDefault();
    select(root, next, true);
  });
  mount();
  document.addEventListener("strategic-page-mounted", mount);
  document.addEventListener("strategic-live-regions-refreshed", mount);
})();
