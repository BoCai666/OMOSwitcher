#!/usr/bin/env python3
"""
Tauri 图标生成脚本 (修复版)
使用正确的 PIL 方法保存多尺寸 ICO
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
ICO_SIZES = [16, 32, 48, 64, 128, 256]

# PNG 尺寸
PNG_SIZES = [32, 64, 128, 256]


def generate_ico_v2(source_img: Image.Image, output_path: str):
    """
    生成多尺寸 ICO 文件
    使用正确的 PIL 方法：sizes 参数接受像素值列表，不是元组
    """
    # 生成各尺寸图像
    icon_images = []

    for size in ICO_SIZES:
        resized = source_img.resize((size, size), Image.Resampling.LANCZOS)
        icon_images.append(resized)
        print(f"  Generated {size}x{size}")

    # 保存多尺寸 ICO
    # PIL ICO 格式要求：第一个图像 + append_images + sizes
    icon_images[0].save(
        output_path,
        format="ICO",
        sizes=[(s, s) for s in ICO_SIZES],
        append_images=icon_images[1:],
    )

    return os.path.getsize(output_path)


def verify_ico(path: str):
    """验证 ICO 文件"""
    img = Image.open(path)
    sizes = img.info.get("sizes", [])
    print(f"\nVerification:")
    print(f"  File: {os.path.basename(path)}")
    print(f"  Sizes in ICO: {len(sizes)}")
    for w, h in sorted(sizes):
        print(f"    {w}x{h}")
    print(f"  File size: {os.path.getsize(path):,} bytes")


def main():
    print("=" * 50)
    print("Tauri Icon Generator")
    print("=" * 50)

    # 检查源文件
    if not os.path.exists(SOURCE_IMAGE):
        print(f"Error: Source image not found: {SOURCE_IMAGE}")
        sys.exit(1)

    # 读取原图
    source_img = Image.open(SOURCE_IMAGE)
    print(f"\nSource: {SOURCE_IMAGE}")
    print(f"  Size: {source_img.size}")
    print(f"  Mode: {source_img.mode}")

    if source_img.mode != "RGBA":
        source_img = source_img.convert("RGBA")
        print(f"  Converted to RGBA")

    # 生成 ICO
    print("\nGenerating ICO:")
    ico_path = os.path.join(ICONS_DIR, "icon.ico")
    ico_size = generate_ico_v2(source_img, ico_path)

    # 验证
    verify_ico(ico_path)

    # 生成 PNG
    print("\nGenerating PNG:")
    for size in PNG_SIZES:
        output_path = os.path.join(ICONS_DIR, f"{size}x{size}.png")
        resized = source_img.resize((size, size), Image.Resampling.LANCZOS)
        resized.save(output_path, "PNG")
        print(f"  {size}x{size}.png")

    # 128x128@2x
    output_path = os.path.join(ICONS_DIR, "128x128@2x.png")
    source_img.resize((256, 256), Image.Resampling.LANCZOS).save(output_path, "PNG")
    print(f"  128x128@2x.png (256x256)")

    print("\n" + "=" * 50)
    print("Done! Rebuild with: npm run tauri:build")
    print("=" * 50)


if __name__ == "__main__":
    main()
