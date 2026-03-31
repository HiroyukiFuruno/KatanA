## 1. 同期契約

- [ ] 1.1 split mode の scroll sync が扱う source / target / logical position / 収束条件を、他の AI エージェントが推測なしで導けるように定義する
- [ ] 1.2 文書末尾、最後の見出し以降の tail 区間、heading 0 件文書の fallback を同じ契約の中で明示する
- [ ] 1.3 同期適用後の pane が即座に逆方向の source にならない条件を定義する

## 2. 同期写像と state の整理

- [ ] 2.1 editor / preview の双方で別々に持っている scroll 補間ロジックを、共有できる mapper または同等の責務分離へ整理する
- [ ] 2.2 heading anchor に加えて EOF を含む対応表を扱えるようにし、tail 区間の末尾同期を成立させる
- [ ] 2.3 geometry snapshot の更新と同期適用の順序を整理し、前フレーム依存の末尾ずれを抑える
- [ ] 2.4 consumer 側の write-back suppression を導入し、上下にガタつく往復同期を止める

## 3. 回帰テスト

- [ ] 3.1 editor -> preview の末尾同期を固定化するテストを追加する
- [ ] 3.2 preview -> editor の末尾同期を固定化するテストを追加する
- [ ] 3.3 最後の見出し以降に長い tail がある文書の同期テストを追加する
- [ ] 3.4 heading がない文書の fallback 同期テストを追加する
- [ ] 3.5 vertical / horizontal split の両方で、数フレーム後に収束しガタつかないことを検証する

## 4. 検証

- [ ] 4.1 split scroll sync が既定有効の状態で従来の分割表示フローを壊していないことを確認する
- [ ] 4.2 末尾同期と no-jitter が手動確認でも再現しないことを確認する
- [ ] 4.3 関連する validation command を実行し、change が apply-ready であることを確認する
