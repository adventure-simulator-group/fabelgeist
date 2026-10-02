import init, {bake_heraldry} from './pkg/adventuresim_heraldry_studio.js';
let ready;
self.onmessage = async ({data}) => {
  if (data.module) { ready = init({module_or_path: data.module}); return; }
  try {
    await ready;
    const result = bake_heraldry(data);
    self.postMessage(result, [result.buffer]);
  } catch (error) { self.postMessage(String(error)); }
};
