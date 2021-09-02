Feature: Pagination Controls

  Scenario: If I run Pagebreak in place, my source files should be updated
    Given I have a "dist/index.html" file with the body:
      """
      <section data-pagebreak="1">
        <p>Item 1</p>
        <p>Item 2</p>
        <p>Item 3</p>
      </section>
      """
    When I run Pagebreak with options:
      | source | output |
      | dist   | dist   |
    Then I should see "Item 1" in "dist/index.html"
    But I should not see "Item 2" in "dist/index.html"
    And I should see "Item 2" in "dist/page/2/index.html"
