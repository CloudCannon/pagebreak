Feature: Pagination Meta

  Scenario: If I specify a data-pagebreak-meta, I should get custom meta tags
    Given I have a "source/index.html" file with the content:
      """
      <html>
      <head>
      <meta name="description" data-pagebreak-meta=":content page :num" content="Free Web tutorials">
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
    Then I should see the file "output/index.html"
    And I should see '<meta content="Free Web tutorials" name="description">' in "output/index.html"
    And I should see the file "output/page/2/index.html"
    And I should see '<meta content="Free Web tutorials page 2" name="description">' in "output/page/2/index.html"