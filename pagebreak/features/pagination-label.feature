Feature: Pagination Labels

	Scenario: If I have pagination labels, they should reflect the page number and total
		Given I have a "source/index.html" file with the body:
			"""
			<section data-pagebreak="1"><p></p><p></p></section>
			<section>
			<p>Page <span data-pagebreak-label="current">1</span> of <span data-pagebreak-label="total">1</span></p>
			</section>
			"""
		When I run Pagebreak
		Then I should see '<p>Page <span>1</span> of <span>2</span></p>' in "output/index.html"
		And I should see '<p>Page <span>2</span> of <span>2</span></p>' in "output/page/2/index.html"
