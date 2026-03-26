<script setup lang="ts">
// 首页视图组件
</script>

<template>
  <div class="home-view">
    <el-container>
      <el-header>
        <h1>OMOSwitcher</h1>
        <p>OhMyOpenCode 模型配置管理工具</p>
      </el-header>
      <el-main>
        <el-card>
          <template #header>
            <span>欢迎使用</span>
          </template>
          <p>请等待 Tauri 后端配置完成后使用完整功能。</p>
          <p>当前为前端开发预览模式。</p>
        </el-card>
      </el-main>
    </el-container>
  </div>
</template>

<style scoped>
.home-view {
  width: 100%;
  height: 100%;
}

.el-container {
  min-height: 100vh;
}

.el-header {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  background-color: #409eff;
  color: white;
}

.el-header h1 {
  margin: 0;
  font-size: 24px;
}

.el-header p {
  margin: 8px 0 0;
  font-size: 14px;
  opacity: 0.8;
}

.el-main {
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 40px;
}

.el-card {
  max-width: 600px;
  width: 100%;
}
</style>
