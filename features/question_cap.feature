Feature: Question Pool Has No Artificial Cap
  As a user writing requirements
  I want the question pool to hold any number of questions
  So that no important questions are dropped

  Background:
    Given a clean working directory

  Scenario: All questions are displayed when there are many
    Given the specwriter is running with a nine-questions mock
    When I type "Build a large application"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    Then the screen should show "Q1 (p9)."
    And the screen should show "Q9 (p1)."
