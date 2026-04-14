import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import { useSyncStore } from '@/stores/sync'

// 预加载 Monitor 组件（避免懒加载延迟）
const MonitorPage = import('@/views/Monitor.vue')

// 本地存储键：是否跳过了登录
const SKIP_LOGIN_KEY = 'omo-skip-login'

// 路由配置
const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Login',
    component: () => import('@/views/LoginView.vue'),
    meta: {
      title: '登录',
      isPublic: true // 公开页面，不需要登录
    }
  },
  {
    path: '/home',
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
    component: () => MonitorPage,
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

// 路由守卫 - 检查登录状态 + 设置页面标题
router.beforeEach(async (to, _from, next) => {
  // 设置页面标题
  document.title = (to.meta.title as string) || 'OMOSwitcher'

  // 公开页面直接放行
  if (to.meta.isPublic) {
    next()
    return
  }

  // 检查是否已登录或已跳过
  const syncStore = useSyncStore()
  const isLoggedIn = syncStore.isLoggedIn
  const hasSkipped = localStorage.getItem(SKIP_LOGIN_KEY) === 'true'

  if (isLoggedIn || hasSkipped) {
    // 已登录或已跳过，允许访问
    next()
  } else {
    // 未登录且未跳过，跳转到登录页
    next('/')
  }
})

export default router
