import '@testing-library/jest-dom';

// jsdom lacks matchMedia (used by framer-motion's useReducedMotion) and
// ResizeObserver (used by the LogDrawer terminal fit). Provide inert stubs so
// components that mount them render in the test environment.
if (typeof window !== 'undefined' && !window.matchMedia) {
  window.matchMedia = (query: string) => ({
    // Treat tests as prefers-reduced-motion so framer-motion swaps content
    // synchronously (jsdom never completes exit animations).
    matches: /prefers-reduced-motion/.test(query),
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  });
}

if (typeof globalThis.ResizeObserver === 'undefined') {
  globalThis.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  };
}
