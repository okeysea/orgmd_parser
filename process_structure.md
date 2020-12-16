# 処理方針

## 基本方針

- 現段階ではMarkdown の記法すべて(commonmark)に対応しない
- 基本的な記法のみの対応でミニマムな実装をしその後拡張していく

## 1 ブロック構造の処理(Document parser 内)

### 見出し

```
<Space>{0..3} '#'{1..6} <Space>{1..}
```

```
(<AnyChar> <softbreak>){1..} ('='{1..}|'-'{1..})
```

一行見だし、2行以上のみだし
一致する場合、'\n'改行またはEOFまで切り出す -> call Headers Parser

pseudocode
```
fn document_parser(s: string) {
  if スペースを検知 {
    if スペースの個数 < 4 {
      if '#' {
      }
    }
  }
}
```


### パラグラフ

それ以外の場合、パラグラフとして`<HardBreak>`またはEOFまで切り出す


## 2 インライン書式(Emphasis)

```
hogehoge *Empasis*hogehoge

hogehoge*Empa
sis*hogehoge
```

見出しにおいても強調のマークダウンが有効であるので、
改行の取り扱いについて難儀していたが、ブロック構造の処理を前処理として
挟むことによって、複数行にわたるもの(ex. paragraph内)と一行におさまるべきもの(ex. 1行見出し)
を区別せずに処理できる。
初期の方針では、改行を含むものとそうでないものの2種類を定義しており美しくなかった。
