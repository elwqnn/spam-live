import { fileURLToPath } from 'node:url';
import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [
		paraglideVitePlugin({
			project: './project.inlang',
			outdir: './.paraglide',
			strategy: ['baseLocale']
		})
	],
	resolve: {
		alias: { $paraglide: fileURLToPath(new URL('./.paraglide', import.meta.url)) }
	},
	test: {
		environment: 'node',
		include: ['src/**/*.test.ts'],
		expect: { requireAssertions: true }
	}
});
