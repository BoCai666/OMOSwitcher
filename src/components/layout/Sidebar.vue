<script setup lang="ts">
// 侧边栏导航组件
// 使用 Element Plus Menu 组件实现应用主导航
import { useRoute, useRouter } from 'vue-router'
import { computed } from 'vue'

const route = useRoute()
const router = useRouter()

// 导航菜单项配置
const menuItems = [
  { index: '/', title: '主页' },
  { index: '/config', title: '模型配置' },
  { index: '/models', title: '模型管理' },
  { index: '/presets', title: '预设管理' }
]

// 计算当前激活的菜单项
const activeIndex = computed(() => route.path)

// 处理菜单选择事件
const handleSelect = (index: string) => {
  router.push(index)
}
</script>

<template>
  <div class="sidebar">
    <el-menu
      :default-active="activeIndex"
      background-color="#304156"
      text-color="#bfcbd9"
      active-text-color="#409eff"
      @select="handleSelect"
    >
      <el-menu-item v-for="item in menuItems" :key="item.index" :index="item.index">
        <span>{{ item.title }}</span>
      </el-menu-item>
    </el-menu>
  </div>
</template>

<style scoped>
.sidebar {
  height: 100%;
}

.el-menu {
  border-right: none;
}
</style>
