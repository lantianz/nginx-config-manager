import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import router from './router';
import './styles/fonts.css'

// 禁用右键菜单
document.addEventListener('contextmenu', (e) => e.preventDefault());

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);

app.mount('#app');
