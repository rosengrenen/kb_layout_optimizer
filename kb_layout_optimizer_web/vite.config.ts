import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    svelte({
      onwarn: (warning) => {
        if (warning.code.startsWith('a11y')) {
          return;
        }
      },
    }),
  ],
});
