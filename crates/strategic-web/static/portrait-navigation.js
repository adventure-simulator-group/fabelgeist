(() => {
  let lifecycle;
  const unmount = () => { lifecycle?.abort(); };
  const mount = () => {
    unmount();
    lifecycle = new AbortController();
    const root = document.getElementById("strategic-page");
    if (!root?.hasAttribute("data-strategic-workspace")) return;
    const syncSelection = () => {
      const links = [...root.querySelectorAll('[data-portrait-tab][href]')];
      const selected = links.filter(link => {
        const href = new URL(link.href).pathname;
        const path = link.dataset.portraitTab === 'profile' ? href.replace(/\/stats$/, '') : href;
        return location.pathname === path || location.pathname.startsWith(`${path}/`);
      }).sort((a, b) => (a.dataset.portraitTab === 'profile') - (b.dataset.portraitTab === 'profile') || b.pathname.length - a.pathname.length)[0];
      links.forEach(link => {
        if (link === selected) link.setAttribute('aria-current', 'page');
        else link.removeAttribute('aria-current');
      });
      root.querySelectorAll('.party-portrait[data-character-id], .party-inventory-portrait').forEach(portrait => {
        portrait.classList.toggle('active', Boolean(selected && portrait.contains(selected)));
      });
      const frame = selected?.closest('.party-portrait');
      const rail = frame?.closest('.party-portrait-overlay');
      if (rail) {
        const bounds = frame.getBoundingClientRect();
        const viewport = rail.getBoundingClientRect();
        if (bounds.right > viewport.right) rail.scrollLeft += bounds.right - viewport.right + 4;
        else if (bounds.left < viewport.left) rail.scrollLeft += bounds.left - viewport.left - 4;
      }
      root.toggleAttribute('data-character-view', Boolean(selected || root.querySelector('.resident-portrait.active')));
      root.querySelectorAll('.settlement-services .nav-tab.active').forEach(tab => {
        tab.setAttribute('aria-current', root.hasAttribute('data-character-view') ? 'location' : 'page');
      });
    };
    syncSelection();
    document.addEventListener('strategic-live-regions-refreshed', syncSelection, { signal: lifecycle.signal });
    document.addEventListener('portrait-view-selected', syncSelection, { signal: lifecycle.signal });

  };
  mount();
  document.addEventListener("strategic-page-mounted", mount);
  document.addEventListener("strategic-page-unmounting", unmount);
})();
