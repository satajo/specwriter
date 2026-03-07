Feature: Question Pool Cap
  As a user writing requirements
  I want the question pool limited to 9 questions
  So that the list stays manageable

  Background:
    Given a clean working directory

  Scenario: No new questions when pool is at capacity
    Given the specwriter is running with a nine-questions mock
    When I type "Build a large application"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Q1 (p9)."
    And the screen should show "Q9 (p1)."
    When I type "Add more features"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Q9 (p1)."
    And the screen should not show "Q10"
