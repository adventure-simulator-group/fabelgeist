import init, {bake_texture} from './pkg/adventuresim_texture_studio.js';
let ready;
self.onmessage = async ({data}) => {
  if (typeof data !== 'string') { ready = init({module_or_path: data.module}); return; }
  try {
    await ready;
    const bytes = bake_texture(data);
    self.postMessage(bytes.buffer, [bytes.buffer]);
  } catch (error) { self.postMessage(String(error)); }
};
