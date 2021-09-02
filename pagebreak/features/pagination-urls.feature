Feature: Pagination URLs

  Scenario: If I specify a data-pagebreak-url, I should get custom URLs
    Given I have a "source/index.html" file with the body:
      """
      <section data-pagebreak="1" data-pagebreak-url="./pb/:num/j/">
        <p>Item 1</p>
        <p>Item 2</p>
      </section>
      """
    When I run Pagebreak
    Then I should see the file "output/index.html"
    And I should see the file "output/pb/2/j/index.html"
    But I should not see the file "output/page/2/index.html"

  Scenario: If I want a complex output location, I can use relative URL paths
    Given I have a "source/red/blue/yellow/index.html" file with the body:
      """
      <section data-pagebreak="1" data-pagebreak-url="../../green/:num/">
        <p>Item 1</p>
        <p>Item 2</p>
      </section>
      """
    When I run Pagebreak
    Then I should see the file "output/red/blue/yellow/index.html"
    And I should see the file "output/red/green/2/index.html"
