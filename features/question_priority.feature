Feature: Question Priority
  As a user reviewing clarifying questions
  I want questions sorted by priority with visual indicators
  So that I can focus on the most impactful questions first

  Background:
    Given a clean working directory

  Scenario: Questions are displayed in priority order (highest first)
    Given SPEC.md already contains "# App\n\n## Questions\n\n### Q1 (p1): Low priority question?\n\nDetails here.\n\n### Q2 (p5): High priority question?\n\nDetails here.\n\n### Q3 (p3): Medium priority question?\n\nDetails here."
    And the specwriter is running with a mock command
    And I switch to the questions tab
    Then the screen should show "[5] High priority question?"
    And "High priority" should appear before "Medium priority" on screen
    And "Medium priority" should appear before "Low priority" on screen

  Scenario: Priority is shown in bracket format
    Given SPEC.md already contains "# App\n\n## Questions\n\n### Q1 (p4): Important question?\n\nWhy does this matter?"
    And the specwriter is running with a mock command
    And I switch to the questions tab
    Then the screen should show "[4] Important question?"

  Scenario: Priority 5 is displayed in bold red
    Given SPEC.md already contains "# App\n\n## Questions\n\n### Q1 (p5): Critical question?\n\nThis is critical.\n\n### Q2 (p2): Other question?\n\nOther."
    And the specwriter is running with a mock command
    When I switch to the questions tab
    And I press Down
    Then the priority indicator "[5]" should be bold red

  Scenario: Priority 4 is displayed in yellow
    Given SPEC.md already contains "# App\n\n## Questions\n\n### Q1 (p5): Top question?\n\nTop.\n\n### Q2 (p4): Important question?\n\nThis is important."
    And the specwriter is running with a mock command
    And I switch to the questions tab
    Then the priority indicator "[4]" should be yellow

  Scenario: Priority 3 and below use default color
    Given SPEC.md already contains "# App\n\n## Questions\n\n### Q1 (p5): Top question?\n\nTop.\n\n### Q2 (p3): Moderate question?\n\nModerate priority."
    And the specwriter is running with a mock command
    And I switch to the questions tab
    Then the priority indicator "[3]" should be in default color

  Scenario: Priorities above 5 are clamped to 5 for display
    Given SPEC.md already contains "# App\n\n## Questions\n\n### Q1 (p9): Legacy question?\n\nFrom old spec."
    And the specwriter is running with a mock command
    And I switch to the questions tab
    Then the screen should show "[5] Legacy question?"
    And the screen should not show "[9]"

  Scenario: Integration produces questions with priority
    Given the specwriter is running with a prioritized mock
    When I type "Build an app"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    Then "core feature" should appear before "target audience" on screen
    And "target audience" should appear before "color scheme" on screen
