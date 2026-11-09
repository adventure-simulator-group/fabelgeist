// Tactical client sound playback. The wasm client calls the window.sfx*
// functions below (crates/adventuresim-tactical-client/src/sound_output.rs).
// The browser decodes each file once with its own decoder, and one
// AudioWorklet (sfx_worklet.js) mixes every voice on the real-time audio
// thread. No decoding or mixing runs on the page's frame loop.
//
// Load this as a classic script, before the client module starts. The client
// calls these functions every frame, and a call to something that is not a
// function ends its frame loop. So they exist before any audio setup runs,
// and they go back to doing nothing if setup throws: broken audio means a
// silent game, not a dead one.
//
// Only a running AudioContext makes sound. Browsers keep it suspended until
// the first click or key press. A one-shot fired while it is suspended is
// dropped rather than queued, so nothing bursts out on unlock; loops pick up
// their current level once it runs.
(() => {
  const silence = () => {
    for (const name of ["sfxPreload", "sfxPlay", "sfxLoop"]) window[name] = () => {};
  };
  silence();
  try {
    const here = document.currentScript.src;
    // The Bevy asset root, which sits beside this script (/tactical/assets).
    const assets = new URL("assets/", here);
    // New one-shots a frame; the rest of the frame's requests are dropped.
    const MAX_STARTS = 16;
    // One-shots at once; over that, a new one replaces the quietest.
    const MAX_VOICES = 48;
    // Distinct loops (wind and the rain beds).
    const MAX_LOOPS = 8;
    const STRIDE = 4; // sfx_worklet.js: sound, left, right, speed
    const TABLE = 1 + MAX_STARTS * STRIDE;
    const frame = new Float32Array(TABLE + MAX_LOOPS);
    const loopGains = frame.subarray(TABLE);
    const postedLoopGains = new Float32Array(MAX_LOOPS).fill(NaN);
    const stats = { requested: 0, started: 0, dropped: 0, loaded: 0, loopUpdates: 0 };

    const context = new AudioContext();
    let mixer = null;
    const mixerReady = context.audioWorklet
      .addModule(new URL("sfx_worklet.js", here))
      .then(() => {
        mixer = new AudioWorkletNode(context, "tactical-sfx-mixer", {
          numberOfInputs: 0,
          numberOfOutputs: 1,
          outputChannelCount: [2],
          processorOptions: { voices: MAX_VOICES, starts: MAX_STARTS, loops: MAX_LOOPS },
        });
        mixer.connect(context.destination);
      });
    mixerReady.catch((error) => console.warn("sfx: no sound:", error));
    const audible = () => mixer !== null && context.state === "running";
    const unlock = () => {
      if (context.state === "suspended") context.resume();
    };
    for (const type of ["pointerdown", "keydown"]) addEventListener(type, unlock, true);

    // One fetch and decode a file. The worklet takes its own copy of the
    // samples; the decoded AudioBuffer is not kept.
    const sounds = new Map();
    const load = (path) => {
      let sound = sounds.get(path);
      if (sound) return sound;
      sound = { id: sounds.size, ready: false };
      sounds.set(path, sound);
      fetch(new URL(path, assets))
        .then((response) => {
          if (!response.ok) throw new Error(`HTTP ${response.status}`);
          return response.arrayBuffer();
        })
        .then((bytes) => context.decodeAudioData(bytes))
        .then(async (buffer) => {
          const channels = [];
          for (let channel = 0; channel < buffer.numberOfChannels; channel++)
            channels.push(buffer.getChannelData(channel).slice());
          await mixerReady;
          mixer.port.postMessage(
            { sound: sound.id, channels },
            channels.map((samples) => samples.buffer),
          );
          sound.ready = true;
          stats.loaded++;
        })
        .catch((error) => console.warn(`sfx: ${path} failed:`, error));
      return sound;
    };

    // The frame's calls are collected and posted to the mixer in one message
    // from a microtask queued by the first of them.
    let starts = 0;
    let flushQueued = false;
    const flush = () => {
      flushQueued = false;
      if (!audible()) {
        starts = 0;
        postedLoopGains.fill(NaN);
        return;
      }
      let changed = starts > 0;
      for (let loop = 0; loop < MAX_LOOPS; loop++)
        changed ||= loopGains[loop] !== postedLoopGains[loop];
      if (changed) {
        frame[0] = starts;
        mixer.port.postMessage(frame);
        postedLoopGains.set(loopGains);
        stats.started += starts;
      }
      starts = 0;
    };
    const queueFlush = () => {
      if (flushQueued) return;
      flushQueued = true;
      queueMicrotask(flush);
    };

    const loops = new Map();
    const loopOf = (path) => {
      let loop = loops.get(path);
      if (loop) return loop;
      if (loops.size === MAX_LOOPS) {
        console.warn(`sfx: no loop slot left for ${path}`);
        return null;
      }
      loop = { slot: loops.size, sound: null, bound: false };
      loops.set(path, loop);
      return loop;
    };

    window.sfxPreload = (path) => {
      load(path);
    };
    // A spatial one-shot: `left` and `right` are the ears' linear gains,
    // `speed` the playback rate (1 as recorded).
    window.sfxPlay = (path, left, right, speed) => {
      stats.requested++;
      const sound = load(path);
      if (!audible() || !sound.ready || starts === MAX_STARTS) {
        stats.dropped++;
        return;
      }
      const at = 1 + starts++ * STRIDE;
      frame[at] = sound.id;
      frame[at + 1] = left;
      frame[at + 2] = right;
      frame[at + 3] = speed;
      queueFlush();
    };
    // A loop's linear gain, every frame. Its file loads the first time it is
    // asked to be heard.
    window.sfxLoop = (path, gain) => {
      stats.loopUpdates++;
      const loop = loopOf(path);
      if (!loop) return;
      if (gain > 0 && loop.sound === null) loop.sound = load(path);
      if (!loop.bound && loop.sound?.ready) {
        mixer.port.postMessage({ loop: loop.slot, sound: loop.sound.id });
        loop.bound = true;
      }
      loopGains[loop.slot] = gain;
      queueFlush();
    };
    window.__sfx = {
      context,
      stats,
      sounds,
      loops,
      get mixer() {
        return mixer;
      },
    };
  } catch (error) {
    console.warn("sfx: no sound:", error);
    silence();
  }
})();
