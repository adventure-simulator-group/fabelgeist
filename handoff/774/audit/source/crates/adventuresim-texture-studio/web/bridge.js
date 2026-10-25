// Browser boundary only. Documents, controls, scheduling, baking and rendering live in Rust.
const storageKey = 'fabelgeist.texture-studio.document';
let pendingImport;
export function load_saved() { try { return localStorage.getItem(storageKey); } catch { return undefined; } }
export function save_document(source) { try { localStorage.setItem(storageKey, source); } catch (error) { console.warn('Autosave unavailable', error); } }
export function library_source() { try { return localStorage.getItem(storageKey + '.library') ?? '{}'; } catch { return '{}'; } }
export function write_library(source) { try { localStorage.setItem(storageKey + '.library', source); return undefined; } catch (error) { return `Preset could not be stored: ${error.message}`; } }
export function choose_import() {
  const input = document.createElement('input'); input.type = 'file'; input.accept = '.json';
  input.onchange = async () => { const file = input.files[0]; if (file) pendingImport = await file.text(); };
  input.click();
}
export function take_import() { const source = pendingImport; pendingImport = undefined; return source; }
export function download_bytes(name, bytes, mime) {
  const url = URL.createObjectURL(new Blob([bytes], {type: mime}));
  const link = document.createElement('a'); link.href = url; link.download = name; link.click();
  setTimeout(() => URL.revokeObjectURL(url), 30000);
}
export function create_bake_worker() {
  const worker = new Worker(new URL('worker.js', document.baseURI), {type: 'module'});
  worker.postMessage({module: window.textureStudioModule});
  return worker;
}
