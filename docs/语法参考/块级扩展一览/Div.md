# Div

**Div**对应于 HTML 之中的 `div` 元素。

参数：

- `1`：内容。

[[块级附加信息]]有效，作用于渲染后最外层元素之上。

## 示例

```example
{{#Div||内容。}}
···
<div><p>内容。</p></div>
```

```example
{! .foo}{{#Div||内容。}}
···
<div class="foo"><p>内容。</p></div>
```
