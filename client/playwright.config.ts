import { defineConfig } from '@playwright/test';

export default defineConfig({
	webServer: [
		{
			command: 'cargo run',
			cwd: '../server',
			url: 'http://127.0.0.1:8080/health',
			reuseExistingServer: !process.env.CI,
			timeout: 120_000
		},
		{
			command: 'npm run dev -- --host 127.0.0.1 --port 4173',
			url: 'http://127.0.0.1:4173',
			reuseExistingServer: !process.env.CI
		}
	],
	testMatch: '**/*.e2e.{ts,js}',
	use: {
		baseURL: 'http://127.0.0.1:4173'
	}
});
