## Context

配布版の実測は `tmp/v0-22-46/installed-startup.log`。高さ19,067pxのPNGが8,192px上限を超え、wgpuのtexture生成でexit 101となる。保存active tabが同じ画像なので再起動でも再現する。

## Goals / Non-Goals

**Goals:** 単独画像・Markdown内画像・再起動復元を同じ上限保護で安全に表示し、正規表現報告を同じv0.22.46で修正する。

**Non-Goals:** GPU上限の引上げ、元ファイルの縮小保存、別repoの修正。

## Decisions

- upload直前に実contextの `max_texture_side` を参照し、超過する画像だけ縦横比を維持して縮小する。画像を拒否する方式では既存タブを表示できないため採用しない。
- 元の寸法を表示状態とtexture cacheへ保持し、画面上のサイズ・比率・操作を変えない。元画像は変更しない。
- 正規表現のエラー色は実際の照合で使用するpattern生成と一致させる。既存の不正入力時の全表示を維持する。
- ユーザーの実入力 `*.png` は通常検索のwildcardとして受け付ける。`*` を含む場合だけ1 path segment内の任意文字へ変換し、basenameのみなら全階層のbasenameへ、`/` を含むなら相対path全体へ一致させる。他の通常文字は従来のsubstring照合、regex modeは標準構文のままとする。
- 一時復旧では36tabsと履歴を保持しactive_pathだけを別の既存タブへ切り替える。公開修正版を直接インストールした後に元PNGをactiveへ戻し、起動検証する。
- 先に回帰テストを失敗させ、その後に修正し、実アプリ起動も検証する。

## Risks / Trade-offs

- 上限超過画像は縮小したtextureになる → 元寸法と比率を保持し、通常画像は無変更でuploadする。
- 最新依存候補の一括診断にはlibcのexact制約競合がある → 実manifestを変更せず、既存診断と公開registryの候補を分けて評価する。
