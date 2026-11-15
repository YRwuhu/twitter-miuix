import { createApp } from 'vue'
import { createPinia } from 'pinia'
import 'miuix-vue/style.css'
import './styles/tokens.css'
import App from './App.vue'
import { initTheme } from './theme'

initTheme()

createApp(App).use(createPinia()).mount('#app')