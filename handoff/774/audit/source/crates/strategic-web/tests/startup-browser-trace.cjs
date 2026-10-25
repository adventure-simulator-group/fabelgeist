// Browser/GPU-process tracing complements renderer-thread CPU samples.
const fs = require("node:fs");
const path = require("node:path");
const { withDeadline } = require("./startup-deadline.cjs");

exports.attach = (cdp, output) => {
  let events = [], active = false, incomplete = false;
  cdp.on("Tracing.dataCollected", event => { if (active) events.push(...event.value); });
  return {
    async start() {
      if (active || incomplete) throw new Error("Cannot restart an active or incomplete browser trace");
      events = []; active = true;
      await cdp.send("Tracing.start", {
        categories: "gpu,gpu.*,dawn,disabled-by-default-gpu.*,disabled-by-default-dawn.*,blink.user_timing",
        options: "record-as-much-as-possible",
      });
    },
    async stop(name) {
      if (!active) return;
      let state, error, onComplete;
      const completed = new Promise(resolve => { onComplete = resolve; cdp.once("Tracing.tracingComplete", onComplete); });
      try {
        state = await withDeadline((async () => {
          await cdp.send("Tracing.end"); return completed;
        })(), "browser trace completion");
      } catch (cause) { error = String(cause); }
      finally { cdp.removeListener("Tracing.tracingComplete", onComplete); active = false; incomplete = !state; }
      fs.writeFileSync(path.join(output, `${name}-browser.json`), JSON.stringify({
        traceEvents: events, complete: Boolean(state),
        dataLossOccurred: state?.dataLossOccurred || false, error,
      }));
      events = [];

    },
  };
};
