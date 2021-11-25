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
    Then I should see a selector 'meta' in "output/index.html" with the attributes:
      | content   | OG Title                   |
      | property  | og:title                   |
    And I should see a selector 'meta' in "output/index.html" with the attributes:
      | content   | Twitter Title              |
      | property  | twitter:title              |
    And I should see a selector 'title' in "output/index.html" with the attributes:
      | innerText | Website Title              |
    # TODO: Gherkin Rust doesn't let us escape pipes in tables:
    And I should see a selector 'meta' in "output/page/2/index.html" with the attributes:
      | content   | OG Title \PIPE Page 2      |
      | property  | og:title                   |
    # TODO: Gherkin Rust doesn't let us escape pipes in tables:
    And I should see a selector 'meta' in "output/page/2/index.html" with the attributes:
      | content   | Twitter Title \PIPE Page 2 |
      | property  | twitter:title              |
    And I should see a selector 'title' in "output/page/2/index.html" with the attributes:
      | innerText | Website Title \PIPE Page 2 |

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
      <p>Item 3</p>
      </section>
      </body>
      </html>
      """
    When I run Pagebreak
    Then I should see a selector 'meta' in "output/index.html" with the attributes:
      | content   | OG Title                |
      | property  | og:title                |
    And I should see a selector 'meta' in "output/index.html" with the attributes:
      | content   | Twitter Title           |
      | property  | twitter:title           |
    And I should see a selector 'title' in "output/index.html" with the attributes:
      | innerText | Website Title           |
    And I should see a selector 'meta' in "output/page/2/index.html" with the attributes:
      | content   | Page 2 of OG Title      |
      | property  | og:title                |
    And I should see a selector 'meta' in "output/page/2/index.html" with the attributes:
      | content   | Page 2 of Twitter Title |
      | property  | twitter:title           |
    And I should see a selector 'title' in "output/page/2/index.html" with the attributes:
      | innerText | Page 2 of Website Title |
    And I should see a selector 'meta' in "output/page/3/index.html" with the attributes:
      | content   | Page 3 of OG Title      |
      | property  | og:title                |
    And I should see a selector 'meta' in "output/page/3/index.html" with the attributes:
      | content   | Page 3 of Twitter Title |
      | property  | twitter:title           |
    And I should see a selector 'title' in "output/page/3/index.html" with the attributes:
      | innerText | Page 3 of Website Title |

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
    Then I should see a selector 'meta' in "output/index.html" with the attributes:
      | content   | OG Title      |
      | property  | og:title      |
    And I should see a selector 'meta' in "output/index.html" with the attributes:
      | content   | Twitter Title |
      | property  | twitter:title |
    And I should see a selector 'title' in "output/index.html" with the attributes:
      | innerText | Website Title |
    And I should see a selector 'meta' in "output/page/2/index.html" with the attributes:
      | content   | OG Title      |
      | property  | og:title      |
    And I should see a selector 'meta' in "output/page/2/index.html" with the attributes:
      | content   | Twitter Title |
      | property  | twitter:title |
    And I should see a selector 'title' in "output/page/2/index.html" with the attributes:
      | innerText | Website Title |
