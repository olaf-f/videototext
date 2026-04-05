import DOMPurify from 'dompurify'
import { marked } from 'marked'

marked.setOptions({
  breaks: true,
  gfm: true,
})

export function renderMarkdown(markdown: string): string {
  if (!markdown.trim()) {
    return '<p class="markdown-placeholder">一键流程完成后，AI 结构化内容会显示在这里。</p>'
  }

  const html = marked.parse(markdown, { async: false }) as string
  return DOMPurify.sanitize(html)
}
