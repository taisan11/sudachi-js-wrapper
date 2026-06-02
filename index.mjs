// ESM wrapper — loads the native CJS binding via createRequire
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const native = require('./index.js')

export const Dictionary = native.Dictionary
export const Dictionary_From_Byte = native.Dictionary_From_Byte
export const DictionaryFromByte = native.DictionaryFromByte
export const dictionaryConfigPaths = native.dictionaryConfigPaths
export const Tokenizer = native.Tokenizer
