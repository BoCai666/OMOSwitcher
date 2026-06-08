import { ref, watch, onUnmounted } from 'vue'

/**
 * 数字平滑过渡：从旧值动画到新值
 * @param source 源值（ref 或 getter）
 * @param options.duration 动画时长（ms），默认 600
 * @param options.easing 缓动函数，默认 easeOutCubic
 */
export function useTweenedNumber(
  source: () => number,
  options: { duration?: number } = {}
) {
  const duration = options.duration ?? 600
  const display = ref(source())
  let frame: number | null = null

  function cancel() {
    if (frame !== null) {
      cancelAnimationFrame(frame)
      frame = null
    }
  }

  watch(source, (newVal) => {
    const oldVal = display.value
    if (oldVal === newVal) return
    const start = performance.now()
    const tick = (now: number) => {
      const t = Math.min((now - start) / duration, 1)
      // easeOutCubic
      const eased = 1 - Math.pow(1 - t, 3)
      display.value = oldVal + (newVal - oldVal) * eased
      if (t < 1) {
        frame = requestAnimationFrame(tick)
      } else {
        display.value = newVal
        frame = null
      }
    }
    cancel()
    frame = requestAnimationFrame(tick)
  })

  onUnmounted(cancel)

  return display
}
