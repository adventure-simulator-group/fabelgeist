// Bound diagnostic RPCs even when a renderer or GPU process stops responding.
exports.withDeadline = async (operation, label, milliseconds = 10_000) => {
  let timer;
  try {
    return await Promise.race([operation, new Promise((_, reject) => {
      timer = setTimeout(() => reject(new Error(`${label} exceeded ${milliseconds} ms`)), milliseconds);
    })]);
  } finally { clearTimeout(timer); }
};
