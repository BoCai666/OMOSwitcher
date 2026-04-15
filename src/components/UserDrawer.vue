<script setup lang="ts">
/**
 * 用户下拉面板
 * 从标题栏头像按钮下方弹出，提供登录/同步/登出功能
 */
import { computed, watch, ref, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import { useSyncStore } from '@/stores/sync'

const props = defineProps<{
  modelValue: boolean
  triggerRef: HTMLButtonElement | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const syncStore = useSyncStore()

// 下拉面板定位
const panelStyle = ref<Record<string, string>>({})

const visible = computed({
  get: () => props.modelValue,
  set: (val: boolean) => emit('update:modelValue', val)
})

const isLoggedIn = computed(() => syncStore.isLoggedIn)
const currentUser = computed(() => syncStore.currentUser)
const isSyncing = computed(() => syncStore.isSyncing)
const lastSyncTime = computed(() => syncStore.lastSyncTime)
const lastError = computed(() => syncStore.lastError)

// 面板打开时计算位置
watch(() => props.modelValue, async (open) => {
  if (open && props.triggerRef) {
    await nextTick()
    const rect = props.triggerRef.getBoundingClientRect()
    panelStyle.value = {
      position: 'fixed',
      top: `${rect.bottom + 6}px`,
      right: `${window.innerWidth - rect.right}px`,
      zIndex: '9999',
    }
  }
})

// 格式化同步时间
const formatSyncTime = (time: string | null | undefined): string => {
  if (!time) return '从未同步'
  const d = new Date(time)
  return d.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// OAuth 登录
async function handleOAuthLogin() {
  try {
    await syncStore.loginWithOAuth()
    ElMessage.success(`已登录为 ${currentUser.value?.login}`)
    visible.value = false
  } catch (e) {
    ElMessage.error(String(e))
  }
}

// 同步操作（自动检测上传或下载）
async function handleSync() {
  try {
    const result = await syncStore.sync()
    if (result.type === 'UpToDate') {
      ElMessage.success('配置已是最新')
    } else if (result.type === 'Uploaded') {
      ElMessage.success(`已上传 ${result.count} 个预设`)
    } else if (result.type === 'Downloaded') {
      ElMessage.success(`已下载 ${result.count} 个预设`)
    } else if (result.type === 'Conflict') {
      ElMessage.warning('检测到同步冲突，请在弹出的对话框中选择')
    }
  } catch (e) {
    ElMessage.error(String(e))
  }
}

// 登出
async function handleLogout() {
  try {
    await syncStore.logout()
    ElMessage.success('已登出')
    visible.value = false
  } catch (e) {
    ElMessage.error(String(e))
  }
}
</script>

<template>
  <Teleport to="body">
    <!-- 点击外部关闭 -->
    <div v-if="visible" class="dropdown-overlay" @click="visible = false" />
    <!-- 下拉面板 -->
    <transition name="dropdown">
      <div v-if="visible" class="dropdown-panel" :style="panelStyle" @click.stop>
        <!-- 未登录 -->
        <template v-if="!isLoggedIn">
          <button class="dropdown-item github-login-item" @click="handleOAuthLogin">
            <svg viewBox="0 0 16 16" fill="currentColor" width="16" height="16">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
            </svg>
            <span>GitHub 登录</span>
          </button>
        </template>

        <!-- 已登录 -->
        <template v-else>
          <!-- 用户信息 -->
          <div class="dropdown-user-info">
            <img
              v-if="currentUser?.avatar_url"
              :src="currentUser.avatar_url"
              :alt="currentUser.login"
              class="dropdown-avatar"
            />
            <div class="dropdown-user-text">
              <span class="dropdown-user-name">{{ currentUser?.name || currentUser?.login }}</span>
              <span class="dropdown-user-login">@{{ currentUser?.login }}</span>
            </div>
          </div>

          <div class="dropdown-divider"></div>

          <!-- 同步状态 -->
          <div class="dropdown-sync-status">
            <span class="sync-label">上次同步</span>
            <span class="sync-value">
              <template v-if="isSyncing">
                <svg class="sync-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 12a9 9 0 11-6.219-8.56"/>
                </svg>
                同步中...
              </template>
              <template v-else>{{ formatSyncTime(lastSyncTime) }}</template>
            </span>
          </div>
          <div v-if="lastError" class="dropdown-sync-error">{{ lastError }}</div>

          <!-- 同步按钮 -->
          <button class="dropdown-item sync-item" :disabled="isSyncing" @click="handleSync">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 12a9 9 0 11-6.219-8.56"/>
              <polyline points="21 3 21 9 15 9"/>
            </svg>
            <span>{{ isSyncing ? '同步中...' : '同步配置' }}</span>
          </button>

          <div class="dropdown-divider"></div>

          <!-- 登出 -->
          <button class="dropdown-item dropdown-item-logout" @click="handleLogout">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4"/>
              <polyline points="16 17 21 12 16 7"/>
              <line x1="21" y1="12" x2="9" y2="12"/>
            </svg>
            <span>登出</span>
          </button>
        </template>
      </div>
    </transition>
  </Teleport>
</template>

<style scoped>
/* ==================== 遮罩层 ==================== */
.dropdown-overlay {
  position: fixed;
  inset: 0;
  z-index: 9998;
}

/* ==================== 下拉面板 ==================== */
.dropdown-panel {
  min-width: 240px;
  max-width: 300px;
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: 10px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12), 0 2px 8px rgba(0, 0, 0, 0.08);
  padding: 6px;
  overflow: hidden;
}

/* ==================== 进入/退出动画 ==================== */
.dropdown-enter-active {
  transition: all 0.15s cubic-bezier(0.16, 1, 0.3, 1);
}
.dropdown-leave-active {
  transition: all 0.1s ease-in;
}
.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-6px) scale(0.96);
}

/* ==================== 通用菜单项 ==================== */
.dropdown-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 9px 12px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s ease;
  background: transparent;
  color: var(--app-text-primary);
}

.dropdown-item:hover {
  background: var(--app-bg-hover);
}

.dropdown-item svg {
  flex-shrink: 0;
}

/* ==================== 用户信息 ==================== */
.dropdown-user-info {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px 10px;
}

.dropdown-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  object-fit: cover;
  flex-shrink: 0;
}

.dropdown-user-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.dropdown-user-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--app-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dropdown-user-login {
  font-size: 12px;
  color: var(--app-text-tertiary);
}

/* ==================== 分割线 ==================== */
.dropdown-divider {
  height: 1px;
  background: var(--app-border-default);
  margin: 4px 8px;
}

/* ==================== 同步状态 ==================== */
.dropdown-sync-status {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 12px;
}

.sync-label {
  font-size: 12px;
  color: var(--app-text-tertiary);
}

.sync-value {
  font-size: 12px;
  color: var(--app-text-secondary);
  display: flex;
  align-items: center;
  gap: 4px;
}

.sync-spin {
  width: 12px;
  height: 12px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.dropdown-sync-error {
  padding: 0 12px 6px;
  font-size: 11px;
  color: var(--app-color-danger);
}

/* ==================== 同步按钮 ==================== */
.sync-item {
  color: var(--app-color-primary);
}

.sync-item:hover:not(:disabled) {
  background: var(--app-bg-hover);
}

.sync-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.sync-item svg {
  width: 14px;
  height: 14px;
}

/* ==================== 登出按钮 ==================== */
.dropdown-item-logout {
  color: var(--app-text-tertiary);
}

.dropdown-item-logout:hover {
  color: var(--app-color-danger);
  background: rgba(239, 68, 68, 0.06);
}

.dropdown-item-logout svg {
  width: 14px;
  height: 14px;
}

/* ==================== GitHub 登录按钮 ==================== */
.github-login-item {
  padding: 10px 12px;
}

.github-login-item svg {
  width: 16px;
  height: 16px;
}

/* ==================== Cyberpunk 主题 ==================== */
html.cyberpunk .dropdown-panel {
  background: rgba(18, 18, 30, 0.97);
  border-color: rgba(0, 255, 255, 0.2);
  box-shadow:
    0 0 30px rgba(0, 255, 255, 0.1),
    0 8px 32px rgba(0, 0, 0, 0.5);
}

html.cyberpunk .dropdown-item:hover {
  background: rgba(0, 255, 255, 0.08);
}

html.cyberpunk .dropdown-divider {
  background: rgba(0, 255, 255, 0.12);
}

html.cyberpunk .dropdown-avatar {
  border: 1.5px solid rgba(0, 255, 255, 0.4);
  box-shadow: 0 0 8px rgba(0, 255, 255, 0.3);
}

html.cyberpunk .sync-item {
  color: #00ffff;
}

html.cyberpunk .sync-item:hover:not(:disabled) {
  background: rgba(0, 255, 255, 0.08);
}

html.cyberpunk .github-login-item {
  color: #00ffff;
}

html.cyberpunk .github-login-item:hover {
  background: rgba(0, 255, 255, 0.1);
}

html.cyberpunk .dropdown-item-logout:hover {
  color: #ff3366;
  background: rgba(255, 51, 102, 0.1);
}

/* ==================== Glassmorphism 主题 ==================== */
html.glassmorphism .dropdown-panel {
  background: rgba(255, 255, 255, 0.92);
  border-color: rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(20px);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.08),
    0 2px 8px rgba(0, 0, 0, 0.04);
}

html.glassmorphism .dropdown-item:hover {
  background: rgba(37, 99, 235, 0.06);
}

html.glassmorphism .dropdown-avatar {
  border: 1.5px solid rgba(37, 99, 235, 0.2);
}

html.glassmorphism .sync-item {
  color: #2563eb;
}

html.glassmorphism .sync-item:hover:not(:disabled) {
  background: rgba(37, 99, 235, 0.06);
}

html.glassmorphism .github-login-item {
  color: #2563eb;
}

html.glassmorphism .github-login-item:hover {
  background: rgba(37, 99, 235, 0.08);
}

html.glassmorphism .dropdown-item-logout:hover {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.06);
}
</style>
