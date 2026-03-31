import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'

// 路由配置
const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Home',
    component: () => import('@/views/Home.vue'),
    meta: {
      title: '首页'
    }
  },
  {
    path: '/config',
    name: 'ModelConfig',
    component: () => import('@/views/ModelConfig.vue'),
    meta: {
      title: '模型配置'
    }
  },
  {
    path: '/models',
    name: 'Models',
    component: () => import('@/views/ModelManage.vue'),
    meta: {
      title: '模型管理'
    }
  },
  {
    path: '/presets',
    name: 'Presets',
    component: () => import('@/views/PresetManage.vue'),
    meta: {
      title: '预设管理'
    }
  },
  {
    path: '/monitor',
    name: 'Monitor',
    component: () => import('@/views/Monitor.vue'),
    meta: {
      title: '监控'
    }
  }
]

// 创建路由实例
const router = createRouter({
  history: createWebHistory(),
  routes
})

// 路由守卫 - 设置页面标题
router.beforeEach((to, _from, next) => {
  document.title = (to.meta.title as string) || 'OMOSwitcher'
  next()
})

export default router
