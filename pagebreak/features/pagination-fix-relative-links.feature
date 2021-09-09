Feature: Pagination Fix Relative Links

  Scenario: By default, existing relative URLs on the page should be fixed for paginated pages
    Given This should be implemented in issue #8
    Given I have a "source/index.html" file with the content:
      """
      <html>
      <head>
      <link rel="stylesheet" href="styles.css" />
      </head>
      <body>
      <a href="contact">Contact</a>
      <section data-pagebreak="1" data-pagebreak-url="./:num/">
      <p>Item 1</p>
      <p>Item 2</p>
      </section>
      </body>
      </html>
      """
    When I run Pagebreak
    Then I should see '<link rel="stylesheet" href="styles.css" />' in "output/index.html"
    And I should see '<a href="contact">Contact</a>' in "output/index.html"
    And I should see '<link rel="stylesheet" href="../styles.css" />' in "output/2/index.html"
    And I should see '<a href="../contact">Contact</a>' in "output/2/index.html"
