#!/usr/bin/env python3
"""
Tauri 图标生成脚本
从一张 1024x1024 的 PNG 图生成所有平台所需的图标
"""

from PIL import Image
import os
import sys

# 路径配置
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
ICONS_DIR = os.path.join(PROJECT_ROOT, "src-tauri", "icons")
SOURCE_IMAGE = os.path.join(ICONS_DIR, "icon.png")

# Windows ICO 需要的尺寸
ICO_SIZES = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]

# PNG 尺寸 (Tauri 要求)
PNG_SIZES = [
    32,  # 32x32.png
    64,  # 64x64.png (任务栏常用)
    128,  # 128x128.png
    256,  # 128x128@2x.png (高 DPI)
]

# Windows Store 尺寸
STORE_SIZES = [30, 44, 71, 89, 107, 142, 150, 284, 310]


def generate_ico(source_img: Image.Image, output_path: str):
    """生成多尺寸 ICO 文件"""
    icon_images = []

    for size in ICO_SIZES:
        resized = source_img.resize(size, Image.Resampling.LANCZOS)
        icon_images.append(resized)
        print(f"  ICO: {size[0]}x{size[1]}")

    # 保存为多尺寸 ICO
    icon_images[0].save(
        output_path, format="ICO", sizes=ICO_SIZES, append_images=icon_images[1:]
    )

    # 验证
    verify = Image.open(output_path)
    actual_sizes = verify.info.get("sizes", [])
    print(f"  -> 已生成 {len(actual_sizes)} 个尺寸")

    return os.path.getsize(output_path)


def generate_png(source_img: Image.Image, icons_dir: str):
    """生成 PNG 图标"""
    generated = []

    for size in PNG_SIZES:
        filename = f"{size}x{size}.png"
        output_path = os.path.join(icons_dir, filename)

        resized = source_img.resize((size, size), Image.Resampling.LANCZOS)
        resized.save(output_path, "PNG")

        generated.append(filename)
        print(f"  PNG: {size}x{size}")

    # 生成 @2x 版本
    resized_256 = source_img.resize((256, 256), Image.Resampling.LANCZOS)
    output_path = os.path.join(icons_dir, "128x128@2x.png")
    resized_256.save(output_path, "PNG")
    generated.append("128x128@2x.png")
    print(f"  PNG: 128x128@2x (256x256)")

    return generated


def main():
    print("=" * 50)
    print("Tauri 图标生成器")
    print("=" * 50)

    # 检查源文件
    if not os.path.exists(SOURCE_IMAGE):
        print(f"错误: 找不到源图像 {SOURCE_IMAGE}")
        sys.exit(1)

    # 读取原图
    source_img = Image.open(SOURCE_IMAGE)
    print(f"\n源图像: {SOURCE_IMAGE}")
    print(f"  尺寸: {source_img.size}")
    print(f"  模式: {source_img.mode}")

    # 确保是 RGBA 模式
    if source_img.mode != "RGBA":
        source_img = source_img.convert("RGBA")
        print(f"  已转换为 RGBA 模式")

    # 生成 ICO
    print("\n生成 ICO (Windows):")
    ico_path = os.path.join(ICONS_DIR, "icon.ico")
    ico_size = generate_ico(source_img, ico_path)
    print(f"  文件大小: {ico_size:,} bytes")

    # 生成 PNG
    print("\n生成 PNG:")
    png_files = generate_png(source_img, ICONS_DIR)
    print(f"  共 {len(png_files)} 个文件")

    print("\n" + "=" * 50)
    print("完成! 请重新构建应用:")
    print("  npm run tauri:build")
    print("=" * 50)


if __name__ == "__main__":
    main()
