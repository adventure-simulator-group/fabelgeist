(() => {
  let lifecycle;
  const unmount = () => lifecycle?.abort();
  const mount = () => {
    unmount();
    lifecycle = new AbortController();
    const root = document.getElementById("strategic-page");
    if (!root) return;
    const nav = root.querySelector(".workspace-bar");
    if (!nav) return;
    const place = root.querySelector(".settlement-services a.active");
    if (place) nav.querySelector("[data-workspace-location]").href = place.href;
    const inventory = root.querySelector(".party-inventory-portrait a[href]");
    const inventoryLink = nav.querySelector("[data-workspace-inventory]");
    if (inventory && inventoryLink) {
      inventoryLink.href = inventory.href;
      inventoryLink.hidden = false;
    }
    nav.querySelectorAll("a[href]").forEach(link => {
      if (new URL(link.href).pathname === location.pathname) link.setAttribute("aria-current", "page");
      else link.removeAttribute("aria-current");
    });
    const chat = root.querySelector(".settlement-chat");
    const toggle = nav.querySelector("[data-workspace-chat]");
    if (chat && toggle) {
      const focusedConversation = Boolean(chat.matches("[data-social-conversation]") || root.querySelector("[data-social-conversation]"));
      const management = Boolean(root.querySelector(".party-member-stage, .service-visual-chest"));
      const saved = sessionStorage.getItem("fabelgeist.conversation-expanded");
      let expanded = focusedConversation || (saved ? saved === "true" : !management);
      const update = () => {
        root.classList.toggle("workspace-chat-closed", !expanded);
        toggle.setAttribute("aria-expanded", String(expanded));
        toggle.textContent = expanded ? "Hide conversation" : "Conversation";
      };
      toggle.hidden = false;
      update();
      toggle.addEventListener("click", () => {
        expanded = !expanded;
        sessionStorage.setItem("fabelgeist.conversation-expanded", String(expanded));
        update();
      }, { signal: lifecycle.signal });
    }
  };
  mount();
  document.addEventListener("strategic-page-mounted", mount);
  document.addEventListener("strategic-page-unmounting", unmount);
})();
