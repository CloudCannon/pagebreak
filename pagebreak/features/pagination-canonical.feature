Feature: Pagination Canonicals

  Scenario: By default, URLs in the meta should be updated to the self page
    Given This should be implemented in issue #3
    Given I have a "source/index.html" file with the content:
      """
      <html>
      <head>
      <link rel="canonical" href="https://example.com/blog/" />
      <meta property="og:url" content="https://example.com/blog/" />
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
    Then I should see '<link rel="canonical" href="https://example.com/blog/" />' in "output/index.html"
    And I should see '<meta property="og:url" content="https://example.com/blog/" />' in "output/index.html"
    Then I should see '<link rel="canonical" href="https://example.com/blog/page/2/" />' in "output/page/2/index.html"
    And I should see '<meta property="og:url" content="https://example.com/blog/page/2/" />' in "output/page/2/index.html"