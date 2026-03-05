// ESM wrapper — loads the native CJS binding via createRequire
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const native = require('./index.js')

export const Dictionary = native.Dictionary
export const Tokenizer = native.Tokenizer
