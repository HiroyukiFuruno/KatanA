# 改善提案: 開発者体験(DX)の向上とAI連携の強化 (機能改善 / 新機能提案)

## 機能改善: Windows環境におけるダイアグラムレンダリングツールのセットアップ自動化
`README.md` の「Appendix: Windows Environment Setup」に記載されている通り、現在Windows環境でMermaidやPlantUMLのダイアグラムを描画するためには、ユーザー自身でNode.js（`mmdc`）やJDKをシステムに手動インストールする必要があります。（PlantUMLの `plantuml.jar` ダウンロード自体はKatanA内で行われますが、ランタイムが別途必要です。）
PlantUMLの場合は軽量なJavaランタイムを内包するか、MermaidについてはRustネイティブ実装への置き換え、あるいはWebAssembly/JavaScriptインタプリタ（Deno/v8等）をKatanAに組み込んでNode.js非依存でレンダリングできるようにすることで、ユーザーの開発者体験を大幅に向上させることができます。

## 新機能提案: AIエージェントワークフローのネイティブ統合
READMEにおいて「Katana × AI Agent = KatanA」というコンセプトが明記されており、"AI-agent-assisted development" のハブとしての役割が期待されています。
現在、外部ツール（`rtk`等）との連携が前提となっていますが、KatanA自身にLLM API（OpenAI, Anthropicなど）やローカルLLM（Ollamaなど）との通信機能を持たせることを提案します。
これにより、
- 仕様書（Markdown）の自動要約
- AIによるコードやUMLダイアグラム（Mermaid/PlantUML）のドラフト生成
- 記述内容に対するAIレビューとインラインチャット

などをエディタ内で完結させることができ、プロダクトのコアビジョンを体現する強力な機能となります。
