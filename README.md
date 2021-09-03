# Pagebreak

> :warning: Pagebreak is in an early stage of development, and breaking changes are likely before a 1.0 release

# Table of contents

- [Pagebreak](#pagebreak)
  - [Intro](#intro)
  - [Page Size](#page-size)
  - [Custom URLs](#custom-urls)
  - [Pagination Controls](#pagination-controls)
  - [Example](#example)
  - [Usage](#usage)

## Intro

Pagebreak is an open-source tool for implementing pagination on any static website output. It is tooling agnostic, and chains onto the end of a complete static site build.

The primary goal of Pagebreak is to decouple pagination logic from templating. Using Pagebreak means you can:
- Configure pagination within a portable component, and paginate whatever page that component winds up on.
- Render components in a component browser, (e.g. with Storybook or Bookshop), without having to mock pagination tags.
- Have editors configure pagination settings on a per-component basis.

## Page Size

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

## Pagination Controls

Next and previous links can be inserted with the `data-pagebreak-control` tag.

```html
<a data-pagebreak-control="prev">Newer Items</a>
<a data-pagebreak-control="next">Older Items</a>
```

Pagebreak will pick these up and update the URLs to link each page to its siblings. In the instance where there is no next or previous page, the element will be removed from the page.

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

## Usage

### Jekyll
`Gemfile`:
```ruby
group :jekyll_plugins do
  gem "jekyll-pagebreak"
end
```
`_config.yml`:
```yml
plugins:
  - jekyll-pagebreak
```
No further configuration is required. Pages containing pagebreak data tags will be discovered and paginated. To use it, simply output your posts as they are, wrapped in a `data-pagebreak` tag. No pagination tags needed. e.g.:

```liquid
<section data-pagebreak="10">
    {% for post in site.posts %}
        {% include post.html post=post %}
    {% endfor %}
</section>
<a href="" data-pagebreak-control="prev">Previous</a>
<a href="" data-pagebreak-control="next">Next</a>
```

### Hugo
// TODO

### Eleventy
// TODO

### Standalone
Pagebreak is also made available as a portable binary, and you can download [the latest release](https://github.com/CloudCannon/pagebreak/releases/latest) for your platform here on GitHub.

When used as a CLI, the flags are:
| Flag      | Short | Description                              |
| --------- | ----- | ---------------------------------------- |
| `source`  | `s`    | Location of a built static website       |
| `output`  | `o`    | Location to output the paginated website |

For example:  
`$: ./pagebreak -s _site -o _site`  
would paginate a website in-place (updating the source files)