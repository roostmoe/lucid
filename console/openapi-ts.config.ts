import { defineConfig } from '@hey-api/openapi-ts';

export default defineConfig({
  input: '../api/openapi.json',
  output: 'src/lib/client',
  plugins: ['@hey-api/client-axios', '@tanstack/react-query'],
});
