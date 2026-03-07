Feature: Tabbed UI
  As a user
  I want to switch between text input and question browsing
  So that I can work efficiently with both free-form input and structured questions

  Background:
    Given a clean working directory
    And the specwriter is running with a mock command

  Scenario: Tab bar is visible on startup
    Then the screen should show "Text input"
    And the screen should show "Open questions"

  Scenario: Text input tab is active by default
    Then the screen should show "Ctrl+S to submit"

  Scenario: Tab key switches to questions tab
    When I press Tab
    Then the screen should show "No open questions"
    And the screen should show "navigate"

  Scenario: Tab key switches back to text input
    When I press Tab
    And I press Tab
    Then the screen should show "Ctrl+S to submit"

  Scenario: Questions tab shows question count in tab name
    Given the spec README already contains "# App\n\n## Questions\n\n### Q1 (p7): First?\n\nBody.\n\n### Q2 (p5): Second?\n\nBody."
    And the specwriter is running with a mock command
    Then the screen should show "Open questions (2)"
