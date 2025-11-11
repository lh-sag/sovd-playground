import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
    server: {
        proxy: {
            '/sovd': 'http://localhost:9000',
        },
    },
    base: '/ui',
    plugins: [vue()],
    build: {
        outDir: '../assets/',
    }
});
