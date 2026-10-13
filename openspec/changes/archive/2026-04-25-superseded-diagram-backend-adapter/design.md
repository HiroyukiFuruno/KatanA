## Context

現行 Mermaid は `mmdc` を探して process 実行し、PNG を返す。PlantUML は `plantuml.jar` を探し、`java -jar` を起動して SVG を返す。どちらも機能しているが、preview code や core rendering code が concrete backend の都合を直接知りやすい。

Rust-native renderer 候補は存在するが、いきなり default にするには syntax parity、visual parity、theme、export、license、packaging の確認が不足している。したがって最初の作業は「backend を差し替え可能にする境界」を作ること。

## Goals / Non-Goals

**Goals:**

- Mermaid / PlantUML を backend adapter 経由で描画する。
- 現行 `mmdc` / Java jar behavior を維持する。
- Rust-native backend を opt-in または spike として評価できるようにする。
- backend failure 時も Markdown preview を維持する。

**Non-Goals:**

- Draw.io backend の置換。
- Rust-native backend を無条件で default にすること。
- WebView、Node embedding、Deno、browser runtime の導入。
- PlantUML server など remote rendering の default 化。

## Decisions

### 1. External backend を最初の adapter implementation にする

最初の実装では現行 `mmdc` と Java jar を adapter の裏側へ移す。ユーザー体験を変えずに依存方向だけを整理する。

### 2. Rust-native backend は parity gate 後に採用する

Mermaid / PlantUML の Rust candidate は、fixture parity、error behavior、theme propagation、export compatibility、license、packaging を通すまで default にしない。

### 3. Preview と export は renderer-neutral result を受け取る

preview / export / cache は concrete backend を知らず、`DiagramResult` 相当の renderer-neutral output を扱う。cache key には backend kind と version を含める。

## Risks / Trade-offs

- [Risk] Adapter が抽象化しすぎる → 初期 contract は Mermaid / PlantUML の現行 behavior に必要な入力へ限定する。
- [Risk] Rust backend の品質が不足する → 外部 backend fallback を残す。
- [Risk] cache invalidation が壊れる → backend id と render options を cache key に含める。

## Migration Plan

1. backend input / output / error contract を定義する。
2. 現行 Mermaid CLI backend を adapter に移す。
3. 現行 PlantUML jar backend を adapter に移す。
4. preview / export call site を adapter output 消費へ寄せる。
5. Rust-native candidate の spike task を切る。

## Open Questions

- Mermaid Rust candidate の評価対象 fixture をどこに置くか。
- PlantUML の Graphviz 依存を bundled runtime として許容するか。
