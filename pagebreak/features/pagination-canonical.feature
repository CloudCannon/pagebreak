Feature: Pagination Canonicals

  Scenario: By default, URLs in the meta should be updated to the self page
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
    Then I should see a selector 'link' in "output/index.html" with the attributes:
      | href     | https://example.com/blog/        |
      | rel      | canonical                        |
    Then I should see a selector 'meta' in "output/index.html" with the attributes:
      | content  | https://example.com/blog/        |
      | property | og:url                           |
    Then I should see a selector 'link' in "output/page/2/index.html" with the attributes:
      | href     | https://example.com/blog/page/2/ |
      | rel      | canonical                        |
    Then I should see a selector 'meta' in "output/page/2/index.html" with the attributes:
      | content  | https://example.com/blog/page/2/ |
      | property | og:url                           |
