# Draw.io 検証用ファイル（fixture）

このディレクトリは、Draw.io の `.drawio` ファイルパターンを集めた検証用ファイル置き場です。

## 構成

- `basic/`: 最小構成や基本要素を確認するために、このリポジトリ用に作成した単純パターン。
- `official/`: diagrams.net 公式サンプル集から取得した複雑なパターン。

## 件数

- `basic/`: 12 ファイル。
- `official/diagrams/`: 8 ファイル。
- `official/examples/`: 73 ファイル。
- `official/blog/`: 99 ファイル。
- `official/training-diagrams/`: 10 ファイル。
- `official/templates/`: 420 ファイル。
- `official/libraries/`: 図形ライブラリ（shape library）XML 3 ファイル。
- `official/template-index/`: テンプレート一覧 XML 1 ファイル。

合計: 622 個の `.drawio` ファイル。

## 出典

- https://www.drawio.com/blog/example-diagrams-github
- https://www.drawio.com/example-diagrams
- https://github.com/jgraph/drawio-diagrams
- https://www.drawio.com/doc/faq/save-file-formats
- https://www.drawio.com/doc/faq/export-to-xml
- https://www.drawio.com/doc/faq/ai-drawio-generation

## 利用条件

`official/` は `jgraph/drawio-diagrams` の `dev` ブランチから取得しています。

- 取得元 commit: `7b70ed6f96738fed4044a25a314733f8544f076e`
- ライセンス: Apache-2.0
- ライセンス全文: `official/LICENSE.apache-2.0.txt`

図面として開ける元 `.xml` は、検証対象の拡張子を揃えるため `.drawio` として保存しています。
図形ライブラリやテンプレート一覧など、`<mxGraphModel>` を直接含まない補助 XML は `.drawio` にせず、`official/libraries/` と `official/template-index/` に分けています。
