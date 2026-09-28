// Pause workers before initialization, sample their CPU, then collect before
// the generation pool terminates them. Opt-in only; production is unchanged.
const fs = require("node:fs");
const path = require("node:path");

exports.attach = async (page, cdp, output) => {
  const pending = new Map(), workers = new Map();
  let nextId = 0, nextWorker = 0;
  cdp.on("Target.receivedMessageFromTarget", ({ sessionId, message }) => {
    const response = JSON.parse(message), key = `${sessionId}:${response.id}`;
    const request = pending.get(key);
    if (!request) return;
    pending.delete(key);
    if (response.error) request.reject(new Error(response.error.message));
    else request.resolve(response.result);
  });
  const send = (sessionId, method, params = {}) => new Promise((resolve, reject) => {
    const id = ++nextId, key = `${sessionId}:${id}`;
    pending.set(key, { resolve, reject });
    cdp.send("Target.sendMessageToTarget", { sessionId, message: JSON.stringify({ id, method, params }) })
      .catch(error => { pending.delete(key); reject(error); });
  });
  cdp.on("Target.attachedToTarget", ({ sessionId, targetInfo }) => {
    const initialized = (async () => {
      if (targetInfo.type === "worker") {
        await send(sessionId, "Profiler.enable");
        await send(sessionId, "Profiler.setSamplingInterval", { interval: 1000 });
        await send(sessionId, "Profiler.start");
      }
      await send(sessionId, "Runtime.runIfWaitingForDebugger");
    })();
    if (targetInfo.type === "worker") workers.set(sessionId, { initialized, index: nextWorker++ });
  });
  await page.exposeFunction("stopStartupWorkers", async () => {
    await Promise.all([...workers].map(async ([sessionId, worker]) => {
      await worker.initialized;
      const { profile } = await send(sessionId, "Profiler.stop");
      fs.writeFileSync(path.join(output, `generation-worker-${worker.index}.cpuprofile`), JSON.stringify(profile));
    }));
    workers.clear();
  });
  await cdp.send("Target.setAutoAttach", { autoAttach: true, waitForDebuggerOnStart: true, flatten: false });
};
