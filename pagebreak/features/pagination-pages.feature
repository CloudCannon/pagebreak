Feature: Pagination Pages

  Scenario: If I have a data tag, I should get multiple pages
    Given I have a "source/index.html" file with the body:
      """
      <section data-pagebreak="1">
        <p>Item 1</p>
        <p>Item 2</p>
      </section>
      """
    When I run Pagebreak
    Then I should see the file "output/index.html"
    And I should see the file "output/page/2/index.html"
    And I should see "Item 1" in "output/index.html"
    And I should see "Item 2" in "output/page/2/index.html"
    But I should not see "Item 2" in "output/index.html"
    And I should not see "Item 1" in "output/page/2/index.html"

  Scenario: If my page size is larger than the number of items, I should still get my first page
    Given I have a "source/index.html" file with the body:
      """
      <section data-pagebreak="10">
        <p>Item 1</p>
        <p>Item 2</p>
      </section>
      """
    When I run Pagebreak
    Then I should see the file "output/index.html"
    And I should see "Item 1" in "output/index.html"
    And I should see "Item 2" in "output/index.html"
    But I should not see the file "output/page/2/index.html"

  Scenario: If my items aren't divisble, my last page should have the remaning items
    Given I have a "source/index.html" file with the body:
      """
      <section data-pagebreak="2">
        <p>Item 1</p>
        <p>Item 2</p>
        <p>Item 3</p>
        <p>Item 4</p>
        <p>Item 5</p>
      </section>
      """
    When I run Pagebreak
    Then I should see the file "output/page/3/index.html"
    And I should see "Item 5" in "output/page/3/index.html"
    But I should not see "Item 4" in "output/page/3/index.html"
