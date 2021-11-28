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
      | source | dist |
      | output | dist |
    Then I should see "Item 1" in "dist/index.html"
    But I should not see "Item 2" in "dist/index.html"
    And I should see "Item 2" in "dist/page/2/index.html"

  Scenario: If I run Pagebreak with a dest, all files should be moved
    Given I have a "source/index.html" file with the body:
      """
      <section data-pagebreak="1">
        <p>Item 1</p>
        <p>Item 2</p>
        <p>Item 3</p>
      </section>
      """
    And I have a "source/extra.txt" file with the body:
      """
      Extra File
      """
    And I have a "source/inner/nested/file.md" file with the body:
      """
      # Inner
      """
    When I run Pagebreak
    Then I should see "Item 1" in "output/index.html"
    And I should see "Extra File" in "output/extra.txt"
    And I should see "# Inner" in "output/inner/nested/file.md"
