Feature: Answer Dialog
  As a user browsing questions
  I want to answer questions directly from the questions tab
  So that I can efficiently address clarifying questions

  Background:
    Given a clean working directory
    Given the spec README already contains "# App\n\n## Questions\n\n### Q1 (p8): Auth requirements?\n\nHow should users authenticate?\n\n### Q2 (p5): Target platform?\n\nWeb, mobile, or desktop?"
    And the specwriter is running with a mock command

  Scenario: Enter opens answer dialog for focused question
    When I switch to the questions tab
    And I press Enter
    Then the screen should show "Answer Q1"

  Scenario: Esc cancels answer dialog
    When I switch to the questions tab
    And I press Enter
    And I type "some text"
    And I press Esc
    Then the screen should not show "Answer Q1"

  Scenario: Submitting answer triggers integration
    When I switch to the questions tab
    And I press Enter
    And I type "OAuth2"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Idle."
