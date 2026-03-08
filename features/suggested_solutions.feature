Feature: Suggested Solutions
  As a user answering questions
  I want to see suggested solutions when available
  So that I can quickly pick a concrete option instead of typing from scratch

  Background:
    Given a clean working directory
    Given SPEC.md already contains "# App\n\n## Questions\n\n### Q1 (p8): What database?\n\nThe app needs storage.\n\n#### PostgreSQL\n\nRobust and reliable.\n\n#### SQLite\n\nLightweight and embedded.\n\n### Q2 (p5): Target platform?\n\nWeb, mobile, or desktop?"
    And the specwriter is running with a mock command

  Scenario: Enter on question with solutions shows solution list
    When I switch to the questions tab
    And I press Enter
    Then the screen should show "PostgreSQL"
    And the screen should show "SQLite"
    And the screen should show "Write custom answer"

  Scenario: Enter on question without solutions opens text input
    When I switch to the questions tab
    And I press Down
    And I press Enter
    Then the screen should show "Type your answer here"
    And the screen should not show "Write custom answer"

  Scenario: Selecting a solution submits it
    When I switch to the questions tab
    And I press Enter
    And I press Enter
    Then the screen should show "Integrating"

  Scenario: Navigating to second solution and selecting it
    When I switch to the questions tab
    And I press Enter
    And I press Down
    And I press Enter
    Then the screen should show "Integrating"

  Scenario: Selecting "Write custom answer" opens text input
    When I switch to the questions tab
    And I press Enter
    And I press Down 2 times
    And I press Enter
    Then the screen should show "Type your answer here"

  Scenario: Esc from solution list closes dialog
    When I switch to the questions tab
    And I press Enter
    Then the screen should show "PostgreSQL"
    When I press Esc
    Then the screen should not show "PostgreSQL"
    And the screen should not show "Answer Q1"

  Scenario: Esc from custom input returns to solution list
    When I switch to the questions tab
    And I press Enter
    And I press Down 2 times
    And I press Enter
    Then the screen should show "Type your answer here"
    When I press Esc
    Then the screen should show "PostgreSQL"
    And the screen should show "SQLite"

  Scenario: Help line in solution selection mode
    When I switch to the questions tab
    And I press Enter
    Then the screen should show "select"
    And the screen should show "navigate"
