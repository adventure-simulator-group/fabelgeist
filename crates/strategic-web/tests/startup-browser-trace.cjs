// Browser/GPU-process tracing complements renderer-thread CPU samples.
const fs = require("node:fs");
const path = require("node:path");

exports.attach = (cdp, output) => {
  let events = [];
  cdp.on("Tracing.dataCollected", event => events.push(...event.value));
  return {
    async start() {
      events = [];
      await cdp.send("Tracing.start", {
        categories: "gpu,gpu.*,dawn,disabled-by-default-gpu.*,disabled-by-default-dawn.*,blink.user_timing",
        options: "record-as-much-as-possible",
      });
    },
    async stop(name) {
      const completed = new Promise(resolve => cdp.once("Tracing.tracingComplete", resolve));
      await cdp.send("Tracing.end");
      const state = await completed;
      fs.writeFileSync(path.join(output, `${name}-browser.json`),
        JSON.stringify({ traceEvents: events, dataLossOccurred: state.dataLossOccurred || false }));
      events = [];
    },
  };
};
