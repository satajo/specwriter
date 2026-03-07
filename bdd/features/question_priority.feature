Feature: Question Priority
  As a user reviewing clarifying questions
  I want questions sorted by priority from high to low
  So that I can focus on the most impactful questions first

  Background:
    Given a clean working directory

  Scenario: Questions are displayed in priority order (highest first)
    Given the spec README already contains "# App\n\n## Questions\n\nQ1 (p3): Low priority question?\n\nQ2 (p9): High priority question?\n\nQ3 (p6): Medium priority question?"
    And the specwriter is running with a mock command
    Then the screen should show "Q2 (p9)."
    And question "Q2" should appear before "Q3" on screen
    And question "Q3" should appear before "Q1" on screen

  Scenario: Priority is displayed alongside each question
    Given the spec README already contains "# App\n\n## Questions\n\nQ1 (p7): Important question?"
    And the specwriter is running with a mock command
    Then the screen should show "Q1 (p7)."

  Scenario: Integration produces questions with priority
    Given the specwriter is running with a prioritized mock
    When I type "Build an app"
    And I press Ctrl+S
    And I wait for integration to complete
    Then question "Q1" should appear before "Q2" on screen
    And question "Q2" should appear before "Q3" on screen
