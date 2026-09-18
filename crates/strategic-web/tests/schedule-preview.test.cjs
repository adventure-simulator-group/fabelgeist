const test = require('node:test');
const assert = require('node:assert/strict');
const { createPreviewRequests } = require('../static/schedule-preview.js');

function harness() {
  const requests = [];
  const states = [];
  let mounted = true;
  const preview = createPreviewRequests((snapshot, signal) => new Promise((resolve, reject) => {
    requests.push({ snapshot, signal, resolve, reject });
  }), { onState: (state) => states.push(state), isMounted: () => mounted });
  return { preview, requests, states, unmount: () => { mounted = false; } };
}

for (const stale of ['success', 'failure']) {
  test(`out-of-order ${stale} cannot replace a newer preview`, async () => {
    const { preview, requests, states } = harness();
    const first = preview.request('old');
    const second = preview.request('new');
    assert.ok(requests[0].signal.aborted);
    requests[1].resolve({ effective: { socializing_minutes: 90 }, leisure_minutes: 1350 });
    await second;
    const latest = states.at(-1);
    if (stale === 'success') requests[0].resolve({ leisure_minutes: 1 });
    else requests[0].reject(new Error('stale server error'));
    await first;
    assert.equal(states.at(-1), latest);
    assert.equal(latest.result.leisure_minutes, 1350);
  });

  test(`an unsubmitted edit immediately invalidates an earlier ${stale}`, async () => {
    const { preview, requests, states } = harness();
    const pending = preview.request('old');
    preview.invalidate();
    assert.ok(requests[0].signal.aborted);
    assert.deepEqual(states.at(-1), { phase: 'editing' });
    if (stale === 'success') requests[0].resolve({ leisure_minutes: 1 });
    else requests[0].reject(new Error('stale server error'));
    await pending;
    assert.deepEqual(states.at(-1), { phase: 'editing' });
  });
}

test('changing mounted character/location suppresses a response', async () => {
  const { preview, requests, states, unmount } = harness();
  const pending = preview.request('draft');
  unmount();
  const count = states.length;
  requests[0].resolve({ effective: {}, leisure_minutes: 1440 });
  await pending;
  assert.equal(states.length, count);
});

test('pending and failed requests expose no old result; a retry can recover', async () => {
  const { preview, requests, states } = harness();
  const first = preview.request('first');
  requests[0].resolve({ leisure_minutes: 10 });
  await first;
  const second = preview.request('second');
  assert.deepEqual(states.at(-1), { phase: 'pending' });
  requests[1].reject(new Error('unavailable'));
  await second;
  assert.equal(states.at(-1).phase, 'error');
  assert.equal(states.at(-1).result, undefined);
  const retry = preview.request('second');
  requests[2].resolve({ leisure_minutes: 20 });
  await retry;
  assert.equal(states.at(-1).result.leisure_minutes, 20);
});

test('unmount disposal aborts work and prevents new requests', async () => {
  const { preview, requests, states } = harness();
  const pending = preview.request('draft');
  preview.dispose();
  const count = states.length;
  assert.ok(requests[0].signal.aborted);
  requests[0].reject(new Error('aborted'));
  await pending;
  await preview.request('new');
  assert.equal(requests.length, 1);
  assert.equal(states.length, count);
});
