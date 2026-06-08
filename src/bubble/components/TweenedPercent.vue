<script setup lang="ts">
import { computed } from 'vue'
import { useTweenedNumber } from '../composables/useTweenedNumber'

const props = defineProps<{
  value: number
}>()

const clamped = computed(() => Math.max(0, Math.min(100, props.value)))
const display = useTweenedNumber(() => clamped.value, { duration: 600 })
</script>

<template>
  <span>{{ Math.round(display) }}%</span>
</template>
