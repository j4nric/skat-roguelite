// The core worker (ADR 0022): the WebAssembly core off the main thread, so
// the page keeps painting and taking taps while an opponent thinks (PIMC).
// One request at a time, in the order they came; every answer carries the
// id of its request (./core.ts).

import { errorMessage, handle, type CoreRequest, type CoreResponse } from './core-engine';

/** The worker's global scope, typed without the WebWorker lib (tsconfig has DOM). */
const scope = self as unknown as {
  onmessage: ((e: MessageEvent<CoreRequest>) => void) | null;
  postMessage(message: CoreResponse): void;
};

// Only `init` is asynchronous; the chain keeps every later request behind it.
let queue: Promise<void> = Promise.resolve();

scope.onmessage = (e) => {
  const { id, op, args } = e.data;
  queue = queue.then(async () => {
    try {
      scope.postMessage({ id, value: await handle(op, args) });
    } catch (err) {
      scope.postMessage({ id, error: errorMessage(err) });
    }
  });
};
