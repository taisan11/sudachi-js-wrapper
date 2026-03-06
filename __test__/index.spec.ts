import { test, expect, describe } from 'bun:test'

// Set SUDACHI_DICT_PATH to the compiled system dictionary (.dic file)
// and optionally SUDACHI_RESOURCE_DIR to the resources directory.
const dictPath = process.env.SUDACHI_DICT_PATH
const resourceDir = process.env.SUDACHI_RESOURCE_DIR

// Dynamic import so missing native binding doesn't crash at module load
const mod = await import('../index.mjs').catch(() => null)
const Dictionary = mod?.Dictionary
const DictionaryFromByte = mod?.Dictionary_From_Byte
const dictionaryConfigPaths = mod?.dictionaryConfigPaths

describe('Dictionary', () => {
  if (!Dictionary) {
    test.skip('(native binding not built — run `bun run build` first)', () => {})
    // eslint-disable-next-line no-useless-return
    return
  }

  test('throws on nonexistent dict path', () => {
    expect(() => new Dictionary('nonexistent.dic')).toThrow()
  })


  test('Dictionary_From_Byte class is exported', () => {
    expect(typeof DictionaryFromByte).toBe('function')
  })

  test('dictionaryConfigPaths returns resolved candidates', () => {
    const info = dictionaryConfigPaths?.('nonexistent.dic')
    expect(info).toBeTruthy()
    expect(info.actualConfigPath).toBeUndefined()
    expect(info.actualConfigExists).toBeUndefined()
    expect(Array.isArray(info.systemDictCandidates)).toBe(true)
    expect(info.systemDictCandidates.length).toBeGreaterThan(0)
  })

  if (dictPath) {
    test('Dictionary_From_Byte can load dictionary bytes', async () => {
      if (!DictionaryFromByte) throw new Error('Dictionary_From_Byte is not available')
      const bytes = await Bun.file(dictPath).bytes()
      const dict = new DictionaryFromByte(Buffer.from(bytes), resourceDir)
      const morphemes = dict.tokenize('東京都に行く')
      expect(morphemes.length).toBeGreaterThan(0)
    })

    test('tokenize returns morphemes', () => {
      const dict = new Dictionary(dictPath, resourceDir)
      const morphemes = dict.tokenize('東京都に行く')
      expect(morphemes.length).toBeGreaterThan(0)
      for (const m of morphemes) {
        expect(typeof m.surface).toBe('string')
        expect(Array.isArray(m.partOfSpeech)).toBe(true)
        expect(typeof m.readingForm).toBe('string')
        expect(typeof m.dictionaryForm).toBe('string')
        expect(typeof m.normalizedForm).toBe('string')
        expect(typeof m.isOov).toBe('boolean')
        expect(typeof m.begin).toBe('number')
        expect(typeof m.end).toBe('number')
        expect(typeof m.dictionaryId).toBe('number')
      }
    })

    test('create() returns a Tokenizer with correct mode', () => {
      const dict = new Dictionary(dictPath, resourceDir)
      const tokenizer = dict.create('C')
      expect(tokenizer.mode).toBe('C')
      const morphemes = tokenizer.tokenize('東京都に行く')
      expect(morphemes.length).toBeGreaterThan(0)
    })

    test('split mode A produces >= morphemes than C', () => {
      const dict = new Dictionary(dictPath, resourceDir)
      const morphemesA = dict.tokenize('東京都に行く', 'A')
      const morphemesC = dict.tokenize('東京都に行く', 'C')
      expect(morphemesA.length).toBeGreaterThanOrEqual(morphemesC.length)
    })
  }
})
