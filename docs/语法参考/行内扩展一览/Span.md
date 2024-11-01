# Span

**Span**对应于 HTML 之中的 `span` 元素。

参数：

- `1`：内容。

[[行内额外语法一览/行内附加信息|行内附加信息]]有效，作用于渲染后最外层元素之上。

## 示例

```example
[{#span|内容}]。
···
<p><span>内容</span>。</p>
```

```example
[! .foo][{#span|内容}]。
···
<p><span class="foo">内容</span>。</p>
```
