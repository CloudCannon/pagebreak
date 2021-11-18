Feature: Pagination Fix Relative Links

  Scenario: By default, existing relative URLs on the page should be fixed for paginated pages
    Given I have a "source/index.html" file with the content:
      """
      <html>
      <head>
      <link rel="stylesheet" href="styles.css" />
      </head>
      <body>
      <a href="contact">Contact</a>
      <section data-pagebreak="1" data-pagebreak-url="./:num/">
      <p>Item 1</p>
      <p>Item 2</p>
      </section>
      </body>
      </html>
      """
    When I run Pagebreak
    Then I should see a selector 'link' in "output/index.html" with the attributes:
      | href      | styles.css    |
      | rel       | stylesheet    |
    Then I should see a selector 'a' in "output/index.html" with the attributes:
      | href      | contact       |
      | innerText | Contact       |
    Then I should see a selector 'link' in "output/2/index.html" with the attributes:
      | href      | ../styles.css |
      | rel       | stylesheet    |
    Then I should see a selector 'a' in "output/2/index.html" with the attributes:
      | href      | ../contact    |
      | innerText | Contact       |

  Scenario: Absolute URLs should remain unchanged in paginated pages
    Given I have a "source/index.html" file with the content:
      """
      <html>
      <head>
      <link rel="stylesheet" href="/styles.css" />
      </head>
      <body>
      <a href="https://placekitten.com/1142/1600">Contact</a>
      <a href="http://placekitten.com/1142/1600">Contract</a>
      <section data-pagebreak="1" data-pagebreak-url="./:num/">
      <p>Item 1</p>
      <p>Item 2</p>
      </section>
      </body>
      </html>
      """
    When I run Pagebreak
    Then I should see a selector 'link' in "output/index.html" with the attributes:
      | href      | /styles.css                       |
      | rel       | stylesheet                        |
    Then I should see a selector 'a' in "output/index.html" with the attributes:
      | href      | https://placekitten.com/1142/1600 |
      | innerText | Contact                           |
    Then I should see a selector 'a' in "output/index.html" with the attributes:
      | href      | http://placekitten.com/1142/1600  |
      | innerText | Contract                          |
    Then I should see a selector 'link' in "output/2/index.html" with the attributes:
      | href      | /styles.css                       |
      | rel       | stylesheet                        |
    Then I should see a selector 'a' in "output/2/index.html" with the attributes:
      | href      | https://placekitten.com/1142/1600 |
      | innerText | Contact                           |
    Then I should see a selector 'a' in "output/2/index.html" with the attributes:
      | href      | http://placekitten.com/1142/1600  |
      | innerText | Contract                          |
