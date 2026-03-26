/**
 * Vue 组件测试示例
 */

import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'

// 创建一个简单的测试组件
const TestComponent = defineComponent({
  name: 'TestComponent',
  setup() {
    return () => h('div', { class: 'test' }, 'Hello Test')
  }
})

describe('Vue 组件测试', () => {
  it('应该正确挂载组件', () => {
    const wrapper = mount(TestComponent)
    expect(wrapper.find('.test').exists()).toBe(true)
    expect(wrapper.text()).toBe('Hello Test')
  })
})
