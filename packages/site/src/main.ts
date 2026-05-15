import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: () => import('./views/Home.vue') },
    { path: '/benchmarks', component: () => import('./views/Benchmarks.vue') },
    { path: '/playground', component: () => import('./views/Playground.vue') },
  ],
})

createApp(App).use(router).mount('#app')
