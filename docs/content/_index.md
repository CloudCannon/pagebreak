---
title: Pagebreak
nav_title: Home
weight: 1
---

Pagebreak is an open-source tool for implementing pagination on any static website output. It is tooling agnostic, and chains onto the end of a complete static site build.

The primary goal of Pagebreak is to decouple pagination logic from templating. Using Pagebreak means you can:
- Configure pagination within a portable component, and paginate whatever page that component winds up on.
- Render components in a component browser, (e.g. with Storybook or Bookshop), without having to mock pagination tags.
- Have editors configure pagination settings on a per-component basis.
