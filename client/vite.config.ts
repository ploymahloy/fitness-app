import tailwindcss from '@tailwindcss/vite';
import vue from '@vitejs/plugin-vue';
import { defineConfig } from 'vite';

export default defineConfig({
	server: {
		proxy: {
			'/data': 'http://localhost:8080',
			'/day': 'http://localhost:8080',
			'/cardio-sessions': 'http://localhost:8080',
			'/weight-sessions': 'http://localhost:8080'
		}
	},
	plugins: [tailwindcss(), vue()]
});
