# Official Draw.io Samples

このディレクトリは、diagrams.net 公式サンプル集から取得した Draw.io ファイルです。

## 出典

- 公式ブログ: https://www.drawio.com/blog/example-diagrams-github
- 公式ギャラリー: https://www.drawio.com/example-diagrams
- 取得元リポジトリ: https://github.com/jgraph/drawio-diagrams

## 取得情報

- branch: `dev`
- commit: `7b70ed6f96738fed4044a25a314733f8544f076e`
- ライセンス: Apache-2.0
- ライセンス全文: `LICENSE.apache-2.0.txt`

## カテゴリ

- `diagrams/`: 公式 README に直接掲載されている代表サンプル。
- `examples/`: UML、BPMN、ネットワーク、インフォグラフィックなどの複雑な例。
- `blog/`: 公式ブログ記事で使われている実例。
- `training-diagrams/`: 公式トレーニング用の例。
- `templates/`: 公式テンプレート（template）集。
- `libraries/`: 図形ライブラリ（shape library）XML。
- `template-index/`: テンプレート一覧 XML。

`templates/` には次のカテゴリがあります。

- `aws`
- `azure`
- `basic`
- `business`
- `charts`
- `education`
- `engineering`
- `flowcharts`
- `gcp`
- `ibm-cloud`
- `infographic`
- `layout`
- `maps`
- `network`
- `other`
- `plans`
- `software`
- `tables`
- `uml`
- `venn`
- `world`

取得元では AWS、Azure、IBM Cloud のテンプレート（template）は `templates/cloud/` に混在していました。
この fixture では GCP と同じ粒度で探せるように、provider 別に `aws/`、`azure/`、`ibm-cloud/` へ分けています。

図面として開ける元 `.xml` は、検証対象の拡張子を揃えるため `.drawio` として保存しています。
図形ライブラリやテンプレート一覧など、`<mxGraphModel>` を直接含まない補助 XML は `.drawio` にしていません。
