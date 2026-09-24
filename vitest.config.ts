import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    // The probe is driven inside its own JSDOM instance per test, so the
    // runner itself needs no DOM environment.
    environment: 'node',
    include: ['guest-js/**/*.test.ts'],
  },
})
