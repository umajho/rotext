# HTML 实体

## 示例

```example
a=&lt; b=['&lt;'] c=`&lt;`
d=&#60; e=&#x3c;
···
<p>a=&lt; b=<strong>&lt;</strong></p> c=<code>&amp;lt;</code><br />d=&#60; e=&#x3c;
```

实体尾部的分号不能被省略：

```example
&lt
···
&amp;lt
```

遇到拥有未知实体名的潜在实体时，其会被视为原封不动的文本：

```example
&thingamabob;
···
<p>&amp;thingamabob;</p>
```
