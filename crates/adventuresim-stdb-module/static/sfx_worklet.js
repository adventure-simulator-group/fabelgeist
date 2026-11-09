// The tactical client's one sound mixer, on the browser's real-time audio
// thread. It decides nothing: which sounds play, how loud and at what speed
// comes from the game through sfx.js. One node mixes every voice, so playing
// a sound creates no Web Audio nodes for the page's garbage collector.
//
// Messages from sfx.js:
//   {sound, channels}  decoded samples of one sound, a Float32Array a channel
//   {loop, sound}      loop slot `loop` plays `sound` from now on
//   Float32Array       one frame: [count, starts x (sound, left, right, speed),
//                      loops x gain]
//
// A one-shot mixes its channels to one and sends that to each ear at the
// ear's own gain, as the game's spatial sounds always have. A loop plays its
// channels as recorded at its gain, and keeps running while silent.
//
// process() must not allocate: everything it touches is made here once.
const STRIDE = 4;

class TacticalSfxMixer extends AudioWorkletProcessor {
  constructor({ processorOptions: { voices, starts, loops } }) {
    super();
    this.table = 1 + starts * STRIDE;
    this.sounds = [];
    this.voiceSound = new Array(voices).fill(null);
    this.voicePosition = new Float64Array(voices);
    this.voiceSpeed = new Float32Array(voices);
    this.voiceLeft = new Float32Array(voices);
    this.voiceRight = new Float32Array(voices);
    this.loopSound = new Array(loops).fill(null);
    this.loopPosition = new Uint32Array(loops);
    this.loopGain = new Float32Array(loops);
    this.loopTarget = new Float32Array(loops);
    this.port.onmessage = ({ data }) => {
      if (data instanceof Float32Array) this.frame(data);
      else if (data.channels) this.sounds[data.sound] = data.channels;
      else this.loopSound[data.loop] = this.sounds[data.sound] ?? null;
    };
  }

  frame(data) {
    for (let i = 0, at = 1; i < data[0]; i++, at += STRIDE) {
      const sound = this.sounds[data[at]];
      if (!sound) continue;
      const voice = this.freeVoice();
      this.voiceSound[voice] = sound;
      this.voicePosition[voice] = 0;
      this.voiceLeft[voice] = data[at + 1];
      this.voiceRight[voice] = data[at + 2];
      this.voiceSpeed[voice] = data[at + 3];
    }
    this.loopTarget.set(data.subarray(this.table, this.table + this.loopTarget.length));
  }

  // An idle voice, or else the quietest playing one, which the new sound
  // replaces.
  freeVoice() {
    let quietest = 0;
    let quietestLevel = Infinity;
    for (let voice = 0; voice < this.voiceSound.length; voice++) {
      if (this.voiceSound[voice] === null) return voice;
      const level = this.voiceLeft[voice] + this.voiceRight[voice];
      if (level < quietestLevel) {
        quietest = voice;
        quietestLevel = level;
      }
    }
    return quietest;
  }

  process(_inputs, outputs) {
    const outLeft = outputs[0][0];
    const outRight = outputs[0][1];
    const frames = outLeft.length;
    for (let voice = 0; voice < this.voiceSound.length; voice++) {
      const channels = this.voiceSound[voice];
      if (channels === null) continue;
      const length = channels[0].length;
      const left = this.voiceLeft[voice];
      const right = this.voiceRight[voice];
      const speed = this.voiceSpeed[voice];
      let position = this.voicePosition[voice];
      let ended = false;
      for (let i = 0; i < frames; i++) {
        const index = position | 0;
        const fraction = position - index;
        let sample = 0;
        for (let channel = 0; channel < channels.length; channel++) {
          const samples = channels[channel];
          const from = samples[index];
          const to = index + 1 < length ? samples[index + 1] : 0;
          sample += from + (to - from) * fraction;
        }
        outLeft[i] += sample * left;
        outRight[i] += sample * right;
        position += speed;
        if (position >= length) {
          ended = true;
          break;
        }
      }
      this.voicePosition[voice] = position;
      if (ended) this.voiceSound[voice] = null;
    }
    for (let loop = 0; loop < this.loopSound.length; loop++) {
      const channels = this.loopSound[loop];
      if (channels === null) continue;
      const leftSamples = channels[0];
      const rightSamples = channels[channels.length > 1 ? 1 : 0];
      const length = leftSamples.length;
      let position = this.loopPosition[loop];
      let gain = this.loopGain[loop];
      const target = this.loopTarget[loop];
      if (gain === 0 && target === 0) {
        this.loopPosition[loop] = (position + frames) % length;
        continue;
      }
      // The new level arrives over one block rather than as a step.
      const step = (target - gain) / frames;
      for (let i = 0; i < frames; i++) {
        gain += step;
        outLeft[i] += leftSamples[position] * gain;
        outRight[i] += rightSamples[position] * gain;
        position = position + 1 === length ? 0 : position + 1;
      }
      this.loopPosition[loop] = position;
      this.loopGain[loop] = target;
    }
    return true;
  }
}

registerProcessor("tactical-sfx-mixer", TacticalSfxMixer);
