## Context

`katana-canvas-forge`（kcf）側で Mermaid 描画、Draw.io 描画、HTML / PDF / PNG / JPEG export の実装を持てる見通しが立っている。

KatanA 側に残る描画・export 実装は利用者向け機能ではなく、実装責務の置き場所の問題である。kcf を取り込んでも画面上の操作や出力形式は変えないため、本 change は version 付き release ではなく `master` 上の内部リファクタリングとして扱う。

## Goals / Non-Goals

**Goals:**

- KatanA 側の Mermaid / Draw.io / export 実装本体を kcf へ寄せる。
- KatanA 側には kcf API を呼び出す薄い adapter だけを残す。
- kcf 更新時に描画 cache が誤って再利用されないようにする。

**Non-Goals:**

- 利用者向けの新しい描画形式や export 形式を追加すること。
- preview UI の見た目や操作を変更すること。
- kcf 側の実装タスクを KatanA 側 OpenSpec に重複して書くこと。

## Decisions

- **version なしで扱う**: kcf intake は利用者に見える機能追加ではないため、`v0.x.0` の release 計画から外し、`canvas-forge-intake` として管理する。
- **master 直作業にする**: OpenSpec と実装のどちらも、release branch ではなく `master` 上で扱う。作業内容は描画 backend の責務整理に限定する。
- **KatanA は adapter に徹する**: KatanA は kcf の `Renderer` / `Exporter` に DTO を渡し、返却結果を既存 preview / export flow に接続する。Mermaid / Draw.io / export の実装本体は KatanA に再作成しない。
- **cache key に kcf runtime 情報を含める**: kcf の `RuntimeVersion` と `RendererProfile` を cache key に含め、kcf release の差分が描画結果へ反映されるようにする。

## Risks / Trade-offs

- **[Risk]** kcf API が未確定のまま KatanA 側 adapter を先に固めると、取り込み時に手戻りが出る。
  - **Mitigation**: `Renderer` / `Exporter` と DTO が確定してから KatanA 側の置換に入る。
- **[Risk]** 実装削除と dependency 追加を同時に行うため、失敗時の切り分けが難しくなる。
  - **Mitigation**: dependency 追加、呼び出し差し替え、旧実装削除を task 単位で分けて検証する。

## Boundaries

```
KatanA preview / export flow
  -> KatanA adapter
  -> katana-canvas-forge Renderer / Exporter
```

- kcf は KatanA UI、`egui`、preview state に依存しない。
- KatanA adapter は kcf DTO と KatanA の既存状態の変換だけを持つ。
- `katana-chat-ui` とは依存関係を持たない。
