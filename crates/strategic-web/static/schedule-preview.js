(() => {
  // Revision invalidation is independent of sending: even an incomplete edit
  // makes every previous success and failure stale. Aborting only saves work.
  function createPreviewRequests(send, { onState, isMounted = () => true }) {
    let revision = 0;
    let active = null;
    let disposed = false;
    const invalidate = () => {
      revision += 1;
      active?.abort();
      active = null;
      if (!disposed && isMounted()) onState({ phase: 'editing' });
    };
    return {
      invalidate,
      async request(snapshot) {
        invalidate();
        if (disposed || !isMounted()) return;
        const requestedRevision = revision;
        const controller = new AbortController();
        active = controller;
        const current = () => !disposed && revision === requestedRevision && isMounted();
        onState({ phase: 'pending' });
        try {
          const result = await send(snapshot, controller.signal);
          if (current()) onState({ phase: 'ready', result });
        } catch (error) {
          if (current()) onState({ phase: 'error', error });
        } finally {
          if (active === controller) active = null;
        }
      },
      dispose() {
        disposed = true;
        invalidate();
      },
    };
  }
  const api = { createPreviewRequests };
  if (typeof module !== 'undefined') module.exports = api;
  if (typeof window !== 'undefined') window.StrategicSchedulePreview = api;
})();
