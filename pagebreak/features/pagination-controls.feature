Feature: Pagination Controls

  Scenario: If I have pagination controls, they should toggle when next/prev pages exist
    Given I have a "source/index.html" file with the body:
      """
      <section data-pagebreak="1">
        <p>Item 1</p>
        <p>Item 2</p>
        <p>Item 3</p>
      </section>
      <section>
        <a href="#" data-pagebreak-control="prev">Previous Page</a>
        <a href="#" data-pagebreak-control="next">Next Page</a>
      </section>
      """
    When I run Pagebreak with options:
      | source | output |
      | source | output |
    Then I should see "Next Page" in "output/index.html"
    And I should see "Next Page" in "output/page/2/index.html"
    And I should see "Previous Page" in "output/page/2/index.html"
    And I should see "Previous Page" in "output/page/3/index.html"
    But I should not see "Previous Page" in "output/index.html"
    And I should not see "Next Page" in "output/page/3/index.html"
