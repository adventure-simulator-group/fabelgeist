(() => {
  const encode = (value) => encodeURIComponent(value).replace(/[!'()*]/g,
    (character) => `%${character.charCodeAt(0).toString(16).toUpperCase()}`);
  const root = ({ kind, id }) => {
    if (kind === "camp") return "/locations/camp";
    if (!id) return null;
    if (kind === "settlement") return `/locations/settlement/${encode(id)}`;
    if (kind === "case_site") return `/locations/case-site/${encode(id)}`;
    return null;
  };
  const parse = (pathname) => {
    const match = /^\/locations\/(settlement|case-site)\/([^/]+)(\/.*)?$/.exec(pathname);
    if (match) {
      try {
        return { kind: match[1] === "case-site" ? "case_site" : "settlement",
          id: decodeURIComponent(match[2]), suffix: match[3] || "" };
      } catch { return null; }
    }
    if (pathname === "/locations/camp" || pathname.startsWith("/locations/camp/")) {
      return { kind: "camp", id: null, suffix: pathname.slice("/locations/camp".length) };
    }
    return null;
  };
  const contains = (pathname, state) => {
    const current = parse(pathname);
    return current !== null && current.kind === state.kind
      && (current.kind === "camp" || current.id === state.id);
  };
  window.strategicLocationUrls = Object.freeze({ root, parse, contains, encode });
})();
