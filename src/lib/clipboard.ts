export async function readImageFromClipboard(): Promise<Blob> {
  if (typeof navigator === 'undefined' || !navigator.clipboard?.read) {
    throw new Error('当前环境不支持读取剪贴板图片。')
  }

  const items = await navigator.clipboard.read()
  for (const item of items) {
    const imageType = item.types.find((type) => type.startsWith('image/'))
    if (imageType) {
      return item.getType(imageType)
    }
  }

  throw new Error('剪贴板中未检测到图片内容。')
}

export async function copyPlainText(text: string): Promise<void> {
  if (typeof navigator === 'undefined' || !navigator.clipboard?.writeText) {
    throw new Error('当前环境不支持写入剪贴板。')
  }

  await navigator.clipboard.writeText(text)
}

export async function copyHtmlAndText(html: string, text: string): Promise<void> {
  if (typeof navigator === 'undefined' || !navigator.clipboard) {
    throw new Error('当前环境不支持写入剪贴板。')
  }

  if (typeof ClipboardItem !== 'undefined' && navigator.clipboard.write) {
    await navigator.clipboard.write([
      new ClipboardItem({
        'text/html': new Blob([html], { type: 'text/html' }),
        'text/plain': new Blob([text], { type: 'text/plain' }),
      }),
    ])
    return
  }

  await copyPlainText(text)
}
