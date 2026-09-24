import { readFileSync } from 'node:fs'

import typescript from '@rollup/plugin-typescript'

const pkg = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf8'))

export default {
  input: 'guest-js/index.ts',
  output: [
    { file: pkg.exports['.'].import, format: 'esm' },
    { file: pkg.exports['.'].require, format: 'cjs' },
  ],
  plugins: [
    typescript({
      tsconfig: './tsconfig.json',
      declaration: true,
      declarationDir: 'dist-js',
      // The probe's tests run in jsdom and read the file from disk; neither
      // belongs in the bundle. The probe itself ships as a plain file, never
      // bundled — bundling it would defeat the point of it being inline ES5.
      exclude: ['**/*.test.ts'],
    }),
  ],
  // `@tauri-apps/api` is a peer of the host app, not something to inline.
  external: [/^@tauri-apps\/api/],
}
