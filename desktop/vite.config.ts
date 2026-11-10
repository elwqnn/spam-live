import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { paraglideVitePlugin } from '@inlang/paraglide-js';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vitest/config';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';

function commit() {
	try {
		return execSync('git rev-parse --short HEAD', { stdio: ['ignore', 'pipe', 'ignore'] })
			.toString()
			.trim();
	} catch {
		return 'dev';
	}
}

process.env.VITE_SITE_URL = (process.env.SPAM_SITE_URL || 'http://localhost:5173').replace(
	/\/+$/,
	''
);
process.env.VITE_SITE_VERSION ||= commit();

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},
			adapter: adapter({ fallback: 'index.html' })
		}),
		paraglideVitePlugin({
			project: '../packages/ui/project.inlang',
			outdir: './src/lib/paraglide',
			emitTsDeclarations: true,
			strategy: ['localStorage', 'preferredLanguage', 'baseLocale']
		})
	],
	resolve: {
		alias: {
			$paraglide: fileURLToPath(new URL('./src/lib/paraglide', import.meta.url)),
			$host: fileURLToPath(new URL('./src/lib/host.ts', import.meta.url))
		}
	},
	build: { assetsInlineLimit: 0 },
	clearScreen: false,
	server: {
		port: 1420,
		strictPort: true,
		watch: { ignored: ['**/src-tauri/**'] }
	},
	test: {
		environment: 'node',
		include: ['src/**/*.test.ts'],
		expect: { requireAssertions: true }
	}
});
