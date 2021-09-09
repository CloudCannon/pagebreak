Feature: Pagination Meta

  Scenario: By default, title tags and title meta elements should be rewritten
    Given I have a "source/index.html" file with the content:
      """
      <html>
      <head>
      <meta property="og:title" content="OG Title">
      <meta property="twitter:title" content="Twitter Title">
      <title>Website Title</title>
      </head>
      <body>
      <section data-pagebreak="1">
      <p>Item 1</p>
      <p>Item 2</p>
      </section>
      </body>
      </html>
      """
    When I run Pagebreak
    Then I should see '<meta content="OG Title" property="og:title">' in "output/index.html"
    And I should see '<meta content="Twitter Title" property="twitter:title">' in "output/index.html"
    And I should see '<title>Website Title</title>' in "output/index.html"
    And I should see '<meta content="OG Title | Page 2" property="og:title">' in "output/page/2/index.html"
    And I should see '<meta content="Twitter Title | Page 2" property="twitter:title">' in "output/page/2/index.html"
    And I should see '<title>Website Title | Page 2</title>' in "output/page/2/index.html"

  Scenario: If I specify a data-pagebreak-meta, I should get custom meta tags
    Given I have a "source/index.html" file with the content:
      """
      <html>
      <head>
      <meta property="og:title" content="OG Title">
      <meta property="twitter:title" content="Twitter Title">
      <title>Website Title</title>
      </head>
      <body>
      <section data-pagebreak="1" data-pagebreak-meta="Page :num of :content">
      <p>Item 1</p>
      <p>Item 2</p>
      </section>
      </body>
      </html>
      """
    When I run Pagebreak
    Then I should see '<meta content="OG Title" property="og:title">' in "output/index.html"
    And I should see '<meta content="Twitter Title" property="twitter:title">' in "output/index.html"
    And I should see '<title>Website Title</title>' in "output/index.html"
    And I should see '<meta content="Page 2 of OG Title" property="og:title">' in "output/page/2/index.html"
    And I should see '<meta content="Page 2 of Twitter Title" property="twitter:title">' in "output/page/2/index.html"
    And I should see '<title>Page 2 of Website Title</title>' in "output/page/2/index.html"

  Scenario: If I don't want to rewrite meta tags, I should set a data-pagebreak-meta=":content"
    Given I have a "source/index.html" file with the content:
      """
      <html>
      <head>
      <meta property="og:title" content="OG Title">
      <meta property="twitter:title" content="Twitter Title">
      <title>Website Title</title>
      </head>
      <body>
      <section data-pagebreak="1" data-pagebreak-meta=":content">
      <p>Item 1</p>
      <p>Item 2</p>
      </section>
      </body>
      </html>
      """
    When I run Pagebreak
    Then I should see '<meta content="OG Title" property="og:title">' in "output/index.html"
    And I should see '<meta content="Twitter Title" property="twitter:title">' in "output/index.html"
    And I should see '<title>Website Title</title>' in "output/index.html"
    And I should see '<meta content="OG Title" property="og:title">' in "output/page/2/index.html"
    And I should see '<meta content="Twitter Title" property="twitter:title">' in "output/page/2/index.html"
    And I should see '<title>Website Title</title>' in "output/page/2/index.html"
