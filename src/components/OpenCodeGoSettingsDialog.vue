<script setup lang="ts">
/**
 * OpenCode Go 额度查询设置对话框
 * 配置网页抓取所需的 id 和 cookie 参数
 * 保存到 OMOSwitcher 的 settings.json（~/.config/omoswitcher/settings.json）
 */
import { ref, watch } from 'vue'
import { getOpenCodeGoConfig, setOpenCodeGoConfig } from '@/services/settingsStore'
import { showSuccess, showError } from '@/utils/errorHandler'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  saved: []
}>()

const formRef = ref()
const loading = ref(false)
const saving = ref(false)

const form = ref({ id: '', cookie: '' })

const rules = {
  id: [{ required: true, message: '请输入订阅 ID', trigger: 'blur' }],
  cookie: [{ required: true, message: '请输入 Cookie', trigger: 'blur' }]
}

watch(() => props.visible, async (val) => {
  if (val) await loadConfig()
})

async function loadConfig() {
  loading.value = true
  try {
    const config = await getOpenCodeGoConfig()
    form.value.id = config.id || ''
    form.value.cookie = config.cookie || ''
  } catch (e) { /* ignore */ }
  finally { loading.value = false }
}

async function handleSave() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  saving.value = true
  try {
    await setOpenCodeGoConfig({ id: form.value.id, cookie: form.value.cookie })
    showSuccess('OpenCode Go 额度查询参数已保存')
    emit('saved')
    emit('update:visible', false)
  } catch (e) {
    showError(String(e))
  } finally { saving.value = false }
}

function handleClose() { emit('update:visible', false) }
</script>

<template>
  <el-dialog
    :model-value="visible"
    title="OpenCode Go 额度查询设置"
    width="560px"
    :close-on-click-modal="true"
    append-to=".app-main"
    align-center
    class="opencode-go-settings-dialog"
    @update:model-value="emit('update:visible', $event)"
  >
    <div class="dialog-body">
      <div v-if="loading" class="loading-state"><el-skeleton :rows="6" animated /></div>
      <template v-else>
        <el-alert title="提示" type="info" :closable="false" show-icon class="settings-tip">
          <template #default>
            <p>OpenCode Go 额度查询通过抓取 Dashboard 页面实现。</p>
            <p>Workspace ID：从 opencode.ai 仪表盘 URL 获取，格式为 opencode.ai/workspace/<b>{id}</b>/go。</p>
            <p>Cookie：浏览器登录 opencode.ai → F12 → Application → Cookies → 复制 auth 的值。</p>
          </template>
        </el-alert>
        <el-form ref="formRef" :model="form" :rules="rules" label-position="top" class="settings-form">
          <el-form-item label="订阅 ID (workspaceId)" prop="id">
            <el-input v-model="form.id" placeholder="从 opencode.ai/workspace/{id}/go 获取" clearable />
          </el-form-item>
          <el-form-item label="Cookie (auth)" prop="cookie">
            <el-input v-model="form.cookie" type="textarea" :rows="4"
              placeholder="浏览器登录 opencode.ai → F12 → Application → Cookies → auth 的值" />
          </el-form-item>
        </el-form>
      </template>
    </div>
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose" :disabled="saving">取消</el-button>
        <el-button type="primary" :loading="saving" @click="handleSave" :disabled="loading">保存</el-button>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped>
.dialog-body { display: flex; flex-direction: column; gap: 20px; }
.loading-state { min-height: 200px; }
.settings-tip { margin: 0; }
.settings-tip :deep(p) { margin: 0 0 4px; line-height: 1.5; font-size: 13px; }
.settings-form { margin: 0; }
.dialog-footer { display: flex; justify-content: flex-end; gap: 12px; }
</style>

<style>
.opencode-go-settings-dialog .el-dialog {
  background: var(--app-bg-card);
  border: 1px solid var(--app-border-default);
  border-radius: 16px;
  min-width: 360px;
  min-height: 400px;
}
.opencode-go-settings-dialog .el-dialog__header { border-bottom: 1px solid var(--app-border-default); padding: 16px 20px; margin: 0; }
.opencode-go-settings-dialog .el-dialog__title { color: var(--app-text-primary); font-weight: 600; font-size: 15px; }
.opencode-go-settings-dialog .el-dialog__body { padding: 20px; }
.opencode-go-settings-dialog .el-dialog__footer { padding: 12px 20px 20px; border-top: 1px solid var(--app-border-default); }
.opencode-go-settings-dialog .el-dialog__headerbtn .el-dialog__close { color: var(--app-text-secondary); }
</style>
