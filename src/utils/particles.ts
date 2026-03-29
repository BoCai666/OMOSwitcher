/**
 * 粒子系统核心逻辑
 * 用于创建赛博朋克风格的粒子动画
 */

export interface ParticleOptions {
  count: number
  color: string
  minSize: number
  maxSize: number
  speed: number
  connectDistance: number
  mouseRadius: number
}

export interface Particle {
  x: number
  y: number
  vx: number
  vy: number
  size: number
  opacity: number
}

const defaultOptions: ParticleOptions = {
  count: 80,
  color: '#00ffff',
  minSize: 1,
  maxSize: 3,
  speed: 0.5,
  connectDistance: 150,
  mouseRadius: 150
}

export class ParticleSystem {
  private canvas: HTMLCanvasElement
  private ctx: CanvasRenderingContext2D
  private particles: Particle[] = []
  private options: ParticleOptions
  private mouseX: number = -1000
  private mouseY: number = -1000
  private animationId: number | null = null
  private running: boolean = false

  constructor(canvas: HTMLCanvasElement, options: Partial<ParticleOptions> = {}) {
    this.canvas = canvas
    this.ctx = canvas.getContext('2d')!
    this.options = { ...defaultOptions, ...options }
    this.init()
  }

  private init(): void {
    this.resize()
    this.createParticles()
    this.bindEvents()
  }

  private createParticles(): void {
    this.particles = []
    for (let i = 0; i < this.options.count; i++) {
      this.particles.push({
        x: Math.random() * this.canvas.width,
        y: Math.random() * this.canvas.height,
        vx: (Math.random() - 0.5) * this.options.speed,
        vy: (Math.random() - 0.5) * this.options.speed,
        size: Math.random() * (this.options.maxSize - this.options.minSize) + this.options.minSize,
        opacity: Math.random() * 0.5 + 0.5
      })
    }
  }

  private bindEvents(): void {
    window.addEventListener('resize', this.resize.bind(this))
    this.canvas.addEventListener('mousemove', this.handleMouseMove.bind(this))
    this.canvas.addEventListener('mouseleave', this.handleMouseLeave.bind(this))
  }

  private resize(): void {
    this.canvas.width = this.canvas.offsetWidth
    this.canvas.height = this.canvas.offsetHeight
  }

  private handleMouseMove(e: MouseEvent): void {
    const rect = this.canvas.getBoundingClientRect()
    this.mouseX = e.clientX - rect.left
    this.mouseY = e.clientY - rect.top
  }

  private handleMouseLeave(): void {
    this.mouseX = -1000
    this.mouseY = -1000
  }

  private update(): void {
    for (const p of this.particles) {
      // 鼠标交互
      const dx = this.mouseX - p.x
      const dy = this.mouseY - p.y
      const dist = Math.sqrt(dx * dx + dy * dy)
      
      if (dist < this.options.mouseRadius) {
        const force = (this.options.mouseRadius - dist) / this.options.mouseRadius
        p.vx -= dx * force * 0.01
        p.vy -= dy * force * 0.01
      }

      // 更新位置
      p.x += p.vx
      p.y += p.vy

      // 边界检测
      if (p.x < 0 || p.x > this.canvas.width) p.vx *= -1
      if (p.y < 0 || p.y > this.canvas.height) p.vy *= -1

      // 速度衰减
      p.vx *= 0.99
      p.vy *= 0.99

      // 最小速度
      const speed = Math.sqrt(p.vx * p.vx + p.vy * p.vy)
      if (speed < 0.1) {
        p.vx = (Math.random() - 0.5) * this.options.speed
        p.vy = (Math.random() - 0.5) * this.options.speed
      }
    }
  }

  private draw(): void {
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height)

    // 将十六进制颜色转换为RGB
    const rgb = this.hexToRgb(this.options.color)

    // 绘制连线
    for (let i = 0; i < this.particles.length; i++) {
      for (let j = i + 1; j < this.particles.length; j++) {
        const p1 = this.particles[i]
        const p2 = this.particles[j]
        const dx = p1.x - p2.x
        const dy = p1.y - p2.y
        const dist = Math.sqrt(dx * dx + dy * dy)

        if (dist < this.options.connectDistance) {
          const opacity = 1 - dist / this.options.connectDistance
          this.ctx.beginPath()
          this.ctx.strokeStyle = `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${opacity * 0.3})`
          this.ctx.lineWidth = 0.5
          this.ctx.moveTo(p1.x, p1.y)
          this.ctx.lineTo(p2.x, p2.y)
          this.ctx.stroke()
        }
      }
    }

    // 绘制粒子
    for (const p of this.particles) {
      this.ctx.beginPath()
      this.ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2)
      this.ctx.fillStyle = `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${p.opacity})`
      this.ctx.fill()
    }
  }

  private hexToRgb(hex: string): { r: number; g: number; b: number } {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
    return result ? {
      r: parseInt(result[1], 16),
      g: parseInt(result[2], 16),
      b: parseInt(result[3], 16)
    } : { r: 0, g: 255, b: 255 }
  }

  private animate(): void {
    if (!this.running) return
    this.update()
    this.draw()
    this.animationId = requestAnimationFrame(this.animate.bind(this))
  }

  start(): void {
    if (this.running) return
    this.running = true
    this.animate()
  }

  stop(): void {
    this.running = false
    if (this.animationId) {
      cancelAnimationFrame(this.animationId)
      this.animationId = null
    }
  }

  destroy(): void {
    this.stop()
    window.removeEventListener('resize', this.resize.bind(this))
    this.canvas.removeEventListener('mousemove', this.handleMouseMove.bind(this))
    this.canvas.removeEventListener('mouseleave', this.handleMouseLeave.bind(this))
  }
}
