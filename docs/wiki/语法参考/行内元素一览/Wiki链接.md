# Wiki链接

**Wiki链接**是导向站内Wiki页面的链接，其主要导向页面。

- 开启：`[[`；
- 闭合：`]]`；
- 槽位：其开启部分与闭合部分之间，首先填充作为页面名称的[[s:通用概念#逐字内容|逐字内容]]。
  - 其次可选：分隔符 `|` 与作为显示名称的[[s:行内阶段#行内序列|行内序列]]槽位。

例如：

```example
[[页面名]]

[[页面名|显示名]]

[[页面名|['显示名']]]
···
<p><x-wiki-link address="页面名"><span slot="content">页面名</span></x-wiki-link></p>
<p><x-wiki-link address="页面名"><span slot="content">显示名</span></x-wiki-link></p>
<p><x-wiki-link address="页面名"><span slot="content"><strong>显示名</strong></span></x-wiki-link></p>
```
