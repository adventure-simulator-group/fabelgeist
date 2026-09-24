(() => {
  let lifecycle;
  const unmount = () => lifecycle?.abort();
  const mount = () => {
    unmount();
    lifecycle = new AbortController();
    document.querySelectorAll('[data-stats-labels]').forEach(control => {
      const panel = control.closest('aside') || control.closest('.sidebar-section');
      if (!panel) return;
      const key = `fabelgeist.stats-labels.${control.dataset.statsLabels}`;
      try { control.checked = localStorage.getItem(key) === 'true'; } catch (_) { /* Session-only when storage is unavailable. */ }
      const update = () => panel.toggleAttribute('data-stat-labels-visible', control.checked);
      update();
      control.addEventListener('change', () => {
        update();
        try { localStorage.setItem(key, String(control.checked)); } catch (_) { /* The current view remains usable. */ }
      }, { signal: lifecycle.signal });
    });
  };
  mount();
  document.addEventListener('strategic-page-mounted', mount);
  document.addEventListener('strategic-page-unmounting', unmount);
})();
