import { describe, expect, it } from 'vitest'

import { appendAccumulatorSection, inferExportBaseName } from './export'

describe('appendAccumulatorSection', () => {
  it('creates a titled section for empty accumulator content', () => {
    expect(appendAccumulatorSection('', 'OCR Result', '  First line\nSecond line  ')).toBe(
      '## OCR Result\n\nFirst line\nSecond line',
    )
  })

  it('appends sections with spacing when accumulator already has content', () => {
    expect(
      appendAccumulatorSection('## OCR Result\n\nFirst line', 'AI Result', '\nStructured text\n'),
    ).toBe('## OCR Result\n\nFirst line\n\n## AI Result\n\nStructured text')
  })

  it('ignores empty body content', () => {
    expect(appendAccumulatorSection('existing', 'AI Result', '   ')).toBe('existing')
  })
})

describe('inferExportBaseName', () => {
  it('drops file extensions and normalizes whitespace', () => {
    expect(inferExportBaseName('Invoice Scan 01.png')).toBe('Invoice Scan 01')
  })

  it('falls back to the app name when there is no display name', () => {
    expect(inferExportBaseName('')).toBe('smartocr-pro')
  })
})
