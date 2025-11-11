import { createApp } from 'vue';
import App from './App.vue';

// bootstrap
import 'bootstrap/dist/css/bootstrap.min.css';
import 'bootstrap-icons/font/bootstrap-icons.css';
import 'bootstrap/dist/js/bootstrap.bundle.min.js';

import './style.css';

const app = createApp(App);

app.config.errorHandler = (err, vm, info) => {
    console.error('Vue Error:', err, info);
};

app.mount('#app');
