# Ankor

我需要为自己开发的网站项目（Rolludejo）引入一类能嵌入在文章（帖子）中的组<wbr />
件，Ankor 即为使用 Solid.js 来构建这类组件的包。

为方便起见，我们将这类组件称为「Ankor <ruby>挂件<rt>Widget</rt></ruby>」。

Ankor 挂件由两部分组成：

- 总是嵌入在文章中的 Label 部分。Label 通常会承载组件的识别性信息。
- 由 Label 延伸出来的 Popper 部分。Popper 通常会承载组件更进一步的信息，<wbr />
  其并不总会呈现在用户眼前。

以刚才提到的网站项目为例，其中的一些 Ankor 挂件包括：

- 引用链接：
  - Label 包括：引用内容的 ID。
  - Popper 包括：引用 ID 指向的实质内容。（还包括用于跳转的按钮等内容。）
- 骰子表达式：
  - Label 包括：输入表达式（过长会被截取并加上省略号），以及求值结果。
  - Popper 包括：求值过程，还有错误时的错误信息。（还包括运行时信息等内容。）
- Wiki链接：
  - Label 包括：目标页面名。
  - Popper 包括：目标页面的实质内容。（还包括用于跳转的按钮等内容。）

Ankor 挂件有以下四种显示模式：

- <ruby>关闭<rt>Closed</rt></ruby>：Popper 完全不显示。
  - 在鼠标点击 Label 时进入固定模式；
  - 在已经打开[^打开]过 Label 后，在鼠标悬置于 Label 之上时进入悬浮模式。
- <ruby>悬浮<rt>Floating</rt></ruby>：Popper 悬浮于 Label 之下，遮盖住 Label
  下方的其他内容。
  - 在鼠标点击 Popper 中的特定图标或 Label 时进入固定模式；
  - 在鼠标从 Label 移至 Popper 之内时仍保持悬浮模式；
  - 在鼠标既不在 Label 也不在 Popper 之内时进入关闭模式。
- <ruby>固定<rt>Pinned</rt></ruby>：Popper 固定于悬浮时的相同位置，Label
  下方内容随之向下挪动，以确保不被挡住。
  - 在鼠标点击 Popper 中的特定图标时进入关闭模式；
  - 在鼠标点击 Label 时进入固定并折叠模式；
- 固定并<ruby>折叠<rt>Collapsed</rt></ruby>：Popper 在固定的同时，超过一<wbr />
  定高度的内容会被隐藏。
  - 在鼠标点击 Popper 中的特定图标时进入关闭模式；
  - 在鼠标点击 Label 或 Popper 中的遮盖时进入固定模式；

[^打开]: 曾处于「悬浮」「固定」或「固定并折叠」模式。

## 大致结构

### 关闭时

```html
<!-- 可以是 `overflow-y: auto;`。 -->
<div class="container">
  <div id="an-ankor-anchor" />
  <article>
    <!-- … -->

    <x-ankor-widget-foo>
      #shadow-root
        <span class="ankor-label-for-foo">Label</span>
    </x-ankor-widget-foo>

    <!-- … -->
  </article>
</div>
```

### 悬浮时

```html
<div class="container">
  <!-- 总是 `position: relative;`。 -->
  <div id="an-ankor-anchor">
    <!-- 在此时是 `position: absolute;`。 -->
    <div class="ankor-popper-for-foo">Popper</div>
  </div>
  <article>
    <!-- … -->

    <x-ankor-widget-foo>
      #shadow-root
        <span class="ankor-label-for-foo">Label</span>
    </x-ankor-widget-foo>

    <!-- … -->
  </article>
</div>
```

### 固定时

```html
<div class="container">
  <div id="an-ankor-anchor" />
  <article>
    <!-- … -->

    <!-- 总是 `display: inline-grid;`。 -->
    <x-ankor-widget-foo>
      #shadow-root
        <span class="ankor-label-for-foo">Label</span>
        <div class="ankor-popper-for-foo">Popper</div>
    </x-ankor-widget-foo>

    <!-- … -->
  </article>
</div>
```

## 用法

TODO
