Feature: Pagination Controls

  Scenario: If I have pagination controls, they should toggle when next/prev pages exist
    Given I have a "source/index.html" file with the body:
      """
      <section data-pagebreak="1"><p></p><p></p><p></p></section>
      <section>
      <a href="#" data-pagebreak-control="prev">Previous Page</a>
      <a href="#" data-pagebreak-control="next">Next Page</a>
      </section>
      """
    When I run Pagebreak
    Then I should see "Next Page" in "output/index.html"
    And I should see "Next Page" in "output/page/2/index.html"
    And I should see "Previous Page" in "output/page/2/index.html"
    And I should see "Previous Page" in "output/page/3/index.html"
    But I should not see "Previous Page" in "output/index.html"
    And I should not see "Next Page" in "output/page/3/index.html"

  Scenario: If I have pagination controls, they should be given the correct hrefs
    Given I have a "source/index.html" file with the body:
      """
      <section data-pagebreak="1"><p></p><p></p><p></p></section>
      <section>
      <a href="#" data-pagebreak-control="prev">Previous Page</a>
      <a href="#" data-pagebreak-control="next">Next Page</a>
      </section>
      """
    When I run Pagebreak
    Then I should see a selector 'a' in "output/index.html" with the attributes:
      | href      | page/2/       |
      | innerText | Next Page     |
    Then I should see a selector 'a' in "output/page/2/index.html" with the attributes:
      | href      | ../3/         |
      | innerText | Next Page     |
    Then I should see a selector 'a' in "output/page/2/index.html" with the attributes:
      | href      | ../../        |
      | innerText | Previous Page |
    Then I should see a selector 'a' in "output/page/3/index.html" with the attributes:
      | href      | ../2/         |
      | innerText | Previous Page |

  Scenario: If I have a complex URL structure, my pagination hrefs should still be correct
    Given I have a "source/red/blue/yellow/index.html" file with the body:
      """
      <section data-pagebreak="1" data-pagebreak-url="../../page/:num/test/"><p></p><p></p><p></p></section>
      <section>
      <a href="#" data-pagebreak-control="prev">Previous Page</a>
      <a href="#" data-pagebreak-control="next">Next Page</a>
      </section>
      """
    When I run Pagebreak
    Then I should see a selector 'a' in "output/red/blue/yellow/index.html" with the attributes:
      | href      | ../../page/2/test/    |
      | innerText | Next Page             |
    Then I should see a selector 'a' in "output/red/page/2/test/index.html" with the attributes:
      | href      | ../../3/test/         |
      | innerText | Next Page             |
    Then I should see a selector 'a' in "output/red/page/2/test/index.html" with the attributes:
      | href      | ../../../blue/yellow/ |
      | innerText | Previous Page         |
    Then I should see a selector 'a' in "output/red/page/3/test/index.html" with the attributes:
      | href      | ../../2/test/         |
      | innerText | Previous Page         |

  Scenario: If I have inverse pagination controls, they should hide when next/prev pages exist
    Given I have a "source/index.html" file with the body:
      """
      <section data-pagebreak="1"><p></p><p></p></section>
      <section>
      <a href="#" data-pagebreak-control="prev">Previous Page</a>
      <span data-pagebreak-control="!prev">No Previous</span>
      <a href="#" data-pagebreak-control="next">Next Page</a>
      <span data-pagebreak-control="!next">No Next</span>
      </section>
      """
    When I run Pagebreak
    Then I should see a selector 'a' in "output/index.html" with the attributes:
      | href      | page/2/       |
      | innerText | Next Page     |
    Then I should see a selector 'span' in "output/index.html" with the attributes:
      | innerText | No Previous   |
    Then I should see a selector 'a' in "output/page/2/index.html" with the attributes:
      | href      | ../../        |
      | innerText | Previous Page |
    Then I should see a selector 'span' in "output/page/2/index.html" with the attributes:
      | innerText | No Next       |
    But I should not see "No Next" in "output/index.html"
    And I should not see "No Previous" in "output/page/2/index.html"