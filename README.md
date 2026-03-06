# @taisan11/sudachi-js-wrapper

![CI](https://github.com/taisan11/sudachi-js-wrapper/actions/workflows/CI.yml/badge.svg)

[sudachi.rs](https://github.com/WorksApplications/sudachi.rs) の Node.js / Bun バインディングです。  
Rust 製の日本語形態素解析器 [Sudachi](https://github.com/WorksApplications/Sudachi) を napi-rs でラップしています。

## 必要なもの

- [Bun](https://bun.sh) または Node.js >= 18
- SudachiDict の辞書ファイル（後述）

## インストール

GitHub Packages からインストールします。  
まず `.npmrc` (または `bunfig.toml`) にレジストリを設定してください：

**`.npmrc`**

```ini
@taisan11:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=YOUR_GITHUB_TOKEN
```

**`bunfig.toml`** (Bun を使う場合)

```toml
[install.scopes]
"@taisan11" = { registry = "https://npm.pkg.github.com", token = "YOUR_GITHUB_TOKEN" }
```

`YOUR_GITHUB_TOKEN` は `read:packages` スコープを持つ [Personal Access Token](https://github.com/settings/tokens) に置き換えてください。

次にインストール：

```bash
# Bun
bun add @taisan11/sudachi-js-wrapper

# npm / yarn
npm install @taisan11/sudachi-js-wrapper
```

## 辞書の準備

Sudachi は別途辞書ファイルが必要です。  
[SudachiDict](https://github.com/WorksApplications/SudachiDict/releases) から `small` / `core` / `full` のいずれかをダウンロードし、解凍してください。

```bash
# 例: core 辞書を取得
curl -L https://github.com/WorksApplications/SudachiDict/releases/latest/download/sudachi-dictionary-latest-core.zip -o dict.zip
unzip dict.zip
# → system_core.dic, char.def, unk.def 等が展開される
```

## 使い方

### ESM (推奨)

```ts
import { Dictionary } from '@taisan11/sudachi-js-wrapper'

const dict = new Dictionary(
  '/path/to/system_core.dic',   // 辞書ファイルのパス（必須）
  '/path/to/resources',         // リソースディレクトリ (char.def 等) （省略可）
)

const morphemes = dict.tokenize('東京都に行く')

for (const m of morphemes) {
  console.log(m.surface, m.partOfSpeech[0], m.readingForm)
}
// 東京都  名詞  トウキョウト
// に      助詞  ニ
// 行く    動詞  イク
```

### CJS

```js
const { Dictionary } = require('@taisan11/sudachi-js-wrapper')

const dict = new Dictionary('/path/to/system_core.dic')
const morphemes = dict.tokenize('東京都に行く')
console.log(morphemes)
```

## API

### `new Dictionary(dictPath, resourceDir?, configPath?)`

| 引数 | 型 | 説明 |
|---|---|---|
| `dictPath` | `string` | コンパイル済み辞書ファイル (`.dic`) のパス |
| `resourceDir` | `string?` | `char.def` / `unk.def` 等を含むリソースディレクトリ |
| `configPath` | `string?` | `sudachi.json` 設定ファイルのパス |

### `dict.tokenize(text, mode?)`

テキストを形態素解析し、`Morpheme[]` を返します。

| 引数 | 型 | デフォルト | 説明 |
|---|---|---|---|
| `text` | `string` | — | 解析対象テキスト |
| `mode` | `"A" \| "B" \| "C"` | `"C"` | 分割モード |

分割モードの違い：

| モード | 粒度 | 例 |
|---|---|---|
| `A` | 短単位 | `東京` / `都` / `に` / `行く` |
| `B` | 中単位 | `東京都` / `に` / `行く` |
| `C` | 長単位（デフォルト）| `東京都` / `に` / `行く` |


### `dictionaryConfigPaths(dictPath?, resourceDir?, configPath?)`

`new Dictionary(...)` と同じ引数を受け取り、実際に Sudachi が使う/探索するパス情報を返します。

返り値には以下を含みます：

- `actualConfigPath`: 実際に使用される config パス
- `actualConfigExists`: その config が存在するか
- `systemDictCandidates`: 辞書パスの探索候補
- `charDefCandidates`: `char.def` の探索候補

### `new Dictionary_From_Byte(dictBytes, resourceDir?, configPath?)`

`.dic` ファイルのバイト列から辞書を構築します。`create` / `tokenize` は `Dictionary` と同じです。

```ts
import { Dictionary_From_Byte } from '@taisan11/sudachi-js-wrapper'

const bytes = await Bun.file('/path/to/system_core.dic').bytes()
const dict = new Dictionary_From_Byte(Buffer.from(bytes), '/path/to/resources')
console.log(dict.tokenize('東京都に行く'))
```

### `dict.create(mode?)`

`Tokenizer` インスタンスを返します。同じモードで繰り返し解析する場合はこちらが効率的です。

```ts
const tokenizer = dict.create('A') // 短単位モードで固定
tokenizer.mode // => "A"
tokenizer.tokenize('東京都に行く')
```

### `Morpheme` オブジェクト

| プロパティ | 型 | 説明 |
|---|---|---|
| `surface` | `string` | 表層形（元テキストの部分文字列） |
| `partOfSpeech` | `string[]` | 品詞情報 6要素 `[品詞, 品詞細分類1, …, 活用型, 活用形]` |
| `readingForm` | `string` | 読み（カタカナ） |
| `dictionaryForm` | `string` | 辞書形（終止形） |
| `normalizedForm` | `string` | 正規化形 |
| `isOov` | `boolean` | 未知語かどうか |
| `begin` | `number` | 元テキスト中の開始バイトオフセット |
| `end` | `number` | 元テキスト中の終了バイトオフセット |
| `dictionaryId` | `number` | 辞書 ID（未知語は `-1`） |

## ローカルでビルドする

```bash
# 依存インストール
bun install

# ネイティブバインディングをビルド（Rust が必要）
bun run build

# テスト（辞書なしでも基本テストは動きます）
bun test

# 辞書ありでテスト
SUDACHI_DICT_PATH=/path/to/system_core.dic \
SUDACHI_RESOURCE_DIR=/path/to/resources \
bun test
```

## リリース

`v` プレフィックス付きのタグを push すると GitHub Actions が自動的に各プラットフォーム向けバイナリをビルドし、GitHub Packages に publish します。

```bash
# バージョンを上げてタグを作成・push
bun run version patch   # または minor / major
git push origin main --tags
```

## ライセンス

MIT © taisan11

本パッケージが使用する [sudachi.rs](https://github.com/WorksApplications/sudachi.rs) および [SudachiDict](https://github.com/WorksApplications/SudachiDict) はそれぞれ Apache 2.0 ライセンスのもとで配布されています。

