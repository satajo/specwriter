Feature: Tabbed UI
  As a user
  I want to switch between text input and question browsing
  So that I can work efficiently with both free-form input and structured questions

  Background:
    Given a clean working directory
    And the specwriter is running with a mock command

  Scenario: Tab bar is visible on startup
    Then the screen should show "Text Input"
    And the screen should show "Open Questions"

  Scenario: Text input tab is active by default
    Then the screen should show "Ctrl+S"

  Scenario: Tab content views have no redundant titles
    Then the screen should not show "Input ("

  Scenario: Tab key switches to questions tab
    When I press Tab
    Then the screen should show "No open questions"
    And the screen should show "navigate"

  Scenario: Tab key cycles back to text input
    When I press Tab
    And I press Tab
    And I press Tab
    Then the screen should show "Ctrl+S"

  Scenario: Questions tab shows question count in tab name
    Given SPEC.md already contains "# App\n\n## Questions\n\n### Q1 (p7): First?\n\nBody.\n\n### Q2 (p5): Second?\n\nBody."
    And the specwriter is running with a mock command
    Then the screen should show "Open Questions (2)"

  Scenario: Focused question details are shown in the detail panel
    Given SPEC.md already contains "# App\n\n## Questions\n\n### Q1 (p8): Auth requirements?\n\nHow should users authenticate?\n\n### Q2 (p5): Target platform?\n\nWeb, mobile, or desktop?"
    And the specwriter is running with a mock command
    When I switch to the questions tab
    Then the screen should show "Details"
    And the detail panel should show "Auth requirements?"
    And the detail panel should show "How should users authenticate?"

  Scenario: Arrow keys change which question details are shown
    Given SPEC.md already contains "# App\n\n## Questions\n\n### Q1 (p8): Auth requirements?\n\nHow should users authenticate?\n\n### Q2 (p5): Target platform?\n\nWeb, mobile, or desktop?"
    And the specwriter is running with a mock command
    When I switch to the questions tab
    Then the detail panel should show "Q1 (p8): Auth requirements?"
    When I press Down
    Then the detail panel should show "Q2 (p5): Target platform?"
