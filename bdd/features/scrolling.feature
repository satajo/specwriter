Feature: Scrolling
  As a user working with large amounts of content
  I want all panels to scroll automatically
  So that I can always see the focused content

  Scenario: Question list scrolls to keep focused question visible
    Given a clean working directory
    And the spec README already contains 20 questions
    And the specwriter is running with a mock command
    When I switch to the questions tab
    And I press Down 15 times
    Then the question list should show "Q16"
    And the detail panel should show "Q16"

  Scenario: Question list scrolls back up
    Given a clean working directory
    And the spec README already contains 20 questions
    And the specwriter is running with a mock command
    When I switch to the questions tab
    And I press Down 15 times
    And I press Up 15 times
    Then the question list should show "Q1"
    And the detail panel should show "Q1"
