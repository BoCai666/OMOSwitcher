<template>
  <canvas ref="canvasRef" class="particle-field"></canvas>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { ParticleSystem } from '@/utils/particles'

const props = defineProps<{
  enabled?: boolean
  count?: number
  color?: string
}>()

const canvasRef = ref<HTMLCanvasElement>()
let particleSystem: ParticleSystem | null = null

onMounted(() => {
  if (canvasRef.value && props.enabled !== false) {
    particleSystem = new ParticleSystem(canvasRef.value, {
      count: props.count ?? 80,
      color: props.color ?? '#00ffff'
    })
    particleSystem.start()
  }
})

onUnmounted(() => {
  particleSystem?.destroy()
})

watch(() => props.enabled, (enabled) => {
  if (enabled && particleSystem) {
    particleSystem.start()
  } else {
    particleSystem?.stop()
  }
})
</script>

<style scoped>
.particle-field {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 0;
}
</style>
