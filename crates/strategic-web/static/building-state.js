(() => {
  let observer;
  const mount = () => {
    observer?.disconnect();
    const page = document.querySelector("#strategic-page");
    if (!page) return;
    const nav = page?.querySelector("[data-settlement-id]");
    const presentation = ["architecturalFamily", "placeSkin", "buildingMaterial"];
    if (!nav) {
      presentation.forEach((name) => delete page.dataset[name]);
      page.style.removeProperty("--active-building-tint");
      return;
    }
    const current = new URL(location.href);
    const requested = current.searchParams.get("building");
    const tabs = [...nav.querySelectorAll("[data-building-id]")];
    const buildings = new Set(tabs.map((tab) => tab.dataset.buildingId).filter(Boolean));
    const requestedPlace = requested !== "map" && buildings.has(requested);
    const serverActive = nav.querySelector(".nav-tab.active")?.dataset.buildingId;
    const context = window.strategicLocationUrls.parse(current.pathname);
    const buildingContextPath = context?.kind === "settlement"
      && /^\/(?:party(?:\/|$)|party-inventory(?:\/|$))/.test(context.suffix);
    const building = buildingContextPath && requestedPlace
      ? requested : (buildings.has(serverActive) ? serverActive : (buildings.has("map") ? "map" : tabs[0]?.dataset.buildingId));
    if (requested && (!requestedPlace || !buildingContextPath)) {
      current.searchParams.delete("building");
      history.replaceState(history.state, "", current);
    }
    tabs.forEach((tab) => {
      const selected = tab.dataset.buildingId === building;
      tab.classList.toggle("active", selected);
      tab.setAttribute("aria-current", selected ? ((buildingContextPath || page.hasAttribute("data-character-view")) ? "location" : "page") : "false");
      if (selected) {
        presentation.forEach((name) => {
          if (tab.dataset[name]) page.dataset[name] = tab.dataset[name];
          else delete page.dataset[name];
        });
        page.style.setProperty("--active-building-tint", tab.style.getPropertyValue("--building-tint"));
      }
    });
    const syncPartyLinks = (root = page) => root.querySelectorAll?.("a[href], form[action]").forEach((node) => {
      const attribute = node.matches("form") ? "action" : "href";
      const raw = node.getAttribute(attribute);
      if (!raw || !raw.startsWith("/locations/")) return;
      const url = new URL(raw, location.origin);
      const target = window.strategicLocationUrls.parse(url.pathname);
      if (target?.kind !== "settlement" || target.id !== nav.dataset.settlementId
        || !/^\/(?:party(?:\/|$)|party-inventory(?:\/|$))/.test(target.suffix)) return;
      if (building && building !== "map") url.searchParams.set("building", building);
      else url.searchParams.delete("building");
      node.setAttribute(attribute, `${url.pathname}${url.search}${url.hash}`);
    });
    syncPartyLinks();
    observer = new MutationObserver((mutations) => mutations.forEach((mutation) =>
      mutation.addedNodes.forEach((node) => node.nodeType === Node.ELEMENT_NODE && syncPartyLinks(node.matches("a[href], form[action]") ? node.parentElement : node)),
    ));
    observer.observe(page, { childList: true, subtree: true });
  };
  mount();
  document.addEventListener("strategic-page-mounted", mount);
  document.addEventListener("strategic-page-unmounting", () => observer?.disconnect());
})();
