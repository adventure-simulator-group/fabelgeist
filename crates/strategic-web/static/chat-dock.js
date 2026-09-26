(() => {
  let lifecycle;
  let layoutObserver;
  const unmount = () => { lifecycle?.abort(); layoutObserver?.disconnect(); };
  const mount = () => {
    unmount();
    const root = document.getElementById('strategic-page');
    const dock = root?.querySelector(':scope > [data-chat-dock]');
    if (!dock) return;
    lifecycle = new AbortController();
    const { signal } = lifecycle;
    // Contextual dialogue supplies the shared dock; menu replacement never owns it.
    const context = root.querySelector('.main-grid .settlement-chat:not(.challenge-chat):not(.challenge-chat-invitation)');
    if (context) dock.replaceChildren(context);
    let observedPanel;
    const align = () => {
      const panel = root.querySelector('.main-grid main.center-content');
      if (panel !== observedPanel) {
        if (observedPanel) layoutObserver.unobserve(observedPanel);
        if (panel) layoutObserver.observe(panel);
        observedPanel = panel;
      }
      const bounds = panel?.getBoundingClientRect();
      const wide = innerWidth > 1100;
      const width = wide ? (bounds?.width || innerWidth / 3) - 24 : innerWidth - 24;
      dock.style.left = `${wide ? (bounds?.left ?? innerWidth / 3) + 12 : 12}px`;
      dock.style.width = `${width}px`;
    };
    layoutObserver = new ResizeObserver(align);
    layoutObserver.observe(root);
    const grid = root.querySelector('.main-grid');
    if (grid) layoutObserver.observe(grid);
    addEventListener('resize', align, { signal });
    document.addEventListener('journal-view-changed', align, { signal });
    align();
  };
  mount();
  document.addEventListener('strategic-page-mounted', mount);
  document.addEventListener('strategic-page-unmounting', unmount);
})();
