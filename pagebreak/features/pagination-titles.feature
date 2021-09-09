Feature: Pagination URLs

  Scenario: If I specify a data-pagebreak-title, I should get custom titles
    Given I have a "source/index.html" file with the content:
      """
      <html>
      <head>
      <title data-pagebreak-title=":num beans :title">My Items</title>
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
    And I should see "<title>My Items</title>" in "output/index.html"
    And I should see the file "output/page/2/index.html"
    And I should see "<title>2 beans My Items</title>" in "output/page/2/index.html"