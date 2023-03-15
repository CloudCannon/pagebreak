---
title: "Getting Started"
nav_title: "Getting Started"
nav_section: Root
weight: 2
---

To paginate a section on your website, add the `data-pagebreak` tag to the container element, and specify how many items you want per page. Then, in your static site generator of choice, output **all** items onto the page.

```html
<section data-pagebreak="2">
    <article>Item 1</article>
    <article>Item 2</article>
    <article>Item 3</article>
</section>
```

Pagebreak will then pick this up, and split the monolithic file into as many pages as needed.

## Custom URLs

By default, for a given `dist/index.html` file, Pagebreak will output pages like the following: `dist/page/2/index.html`

You can customise this url with the `data-pagebreak-url` tag:

```html
<section 
    data-pagebreak="1" 
    data-pagebreak-url="./archive/page-:num/" >
    <article>Item 1</article>
    <article>Item 2</article>
    <article>Item 3</article>
</section>
```

For a given `dist/index.html` file, this would output:
- Page 1: `dist/index.html`
- Page 2: `dist/archive/page-2/index.html`
- Page 3: `dist/archive/page-3/index.html`

The url will be resolved relative to the html file that is being paginated.

## Updating Title & Meta Tags
By default, Pagebreak will update the `<title>` element on paginated pages, as well as the `og:title` and `twitter:title` meta elements. For a given title "Blog", the default page titles will be of the form "Blog | Page 2".

You can customise this title with the `data-pagebreak-meta` tag:

```html
<head>
    <title>Blog</title>
</head>
<body>
    <section
        data-pagebreak="1"
        data-pagebreak-meta=":content Page #:num">
        <article>Item 1</article>
        <article>Item 2</article>
    </section>
</body>
```

With the above example, page 2 would now contain `<title>Blog Page #2</title>`. The first page will always remain unchanged. 

## Pagination Controls
Pagination controls are implemented with the `data-pagebreak-control` attribute.

### Next / Previous Links
Next and previous links can be inserted with the `prev` and `next` controls.

```html
<a data-pagebreak-control="prev">Newer Items</a>
<a data-pagebreak-control="next">Older Items</a>
```

Pagebreak will pick these up and update the URLs to link each page to its siblings. In the instance where there is no next or previous page, the element will be removed from the page.

### Disable Controls
If you want to toggle behavior when a next or previous page doesn't exist, you can use the `!prev` and `!next` controls.

```html
<span data-pagebreak-control="!prev">No Previous Page</span>
<span data-pagebreak-control="!next">No Next Page</span>
```

These elements will be removed from the page if their respective pages exist. 

### Page Numbering
If you want to show current and total page counts, you can use the `current` and `total` controls.

```html
<p>
    Page 
    <span data-pagebreak-label="current">1</span>
    of
    <span data-pagebreak-label="total">1</span>
</p>
```

## Example

Given an `items/index.html` file:
```html
<section 
    data-pagebreak="2" 
    data-pagebreak-url="./page/:num/archive/" >
    <article>Item 1</article>
    <article>Item 2</article>
    <article>Item 3</article>
    <article>Item 4</article>
    <article>Item 5</article>
</section>
<a href="" data-pagebreak-control="prev">Previous</a>
<a href="" data-pagebreak-control="next">Next</a>
```

The new `items/index.html` will look like:
```html
<section>
    <article>Item 1</article>
    <article>Item 2</article>
</section>

<a href="./page/2/archive/">Next</a>
```

And `items/page/2/archive/index.html` will look like:
```html
<section>
    <article>Item 3</article>
    <article>Item 4</article>
</section>
<a href="../../../">Previous</a>
<a href="../../3/archive/">Next</a>
```