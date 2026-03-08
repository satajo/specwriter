Feature: Answer Dialog
  As a user browsing questions
  I want to answer questions directly from the questions tab
  So that I can efficiently address clarifying questions

  Background:
    Given a clean working directory
    Given SPEC.md already contains "# App\n\n## Questions\n\n### Q1 (p5): Auth requirements?\n\nHow should users authenticate?\n\n### Q2 (p3): Target platform?\n\nWeb, mobile, or desktop?"
    And the specwriter is running with a mock command

  Scenario: Enter opens answer dialog for focused question
    When I switch to the questions tab
    And I press Enter
    Then the screen should show "Answer: Auth requirements?"

  Scenario: Esc cancels answer dialog
    When I switch to the questions tab
    And I press Enter
    And I type "some text"
    And I press Esc
    Then the screen should not show "Answer:"

  Scenario: Submitting answer triggers integration
    When I switch to the questions tab
    And I press Enter
    And I type "OAuth2"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Idle."

  Scenario: Answered question is immediately removed from the list
    When I switch to the questions tab
    Then the screen should show "[5] Auth requirements?"
    When I press Enter
    And I type "OAuth2"
    And I press Ctrl+S
    Then the screen should not show "Auth requirements?"
    And the screen should show "[3] Target platform?"

  Scenario: Answer dialog shows placeholder text when empty
    When I switch to the questions tab
    And I press Enter
    Then the screen should show "Type your answer here. Ctrl+S to submit."

  Scenario: Answer dialog placeholder disappears when typing
    When I switch to the questions tab
    And I press Enter
    And I type "x"
    Then the screen should not show "Type your answer here"

  Scenario: Empty answer submission is ignored
    When I switch to the questions tab
    And I press Enter
    And I press Ctrl+S
    Then the screen should show "Answer: Auth requirements?"
    And the screen should show "Idle."

  Scenario: Focus moves to next question after answering
    When I switch to the questions tab
    Then the screen should show "[5] Auth requirements?"
    When I press Enter
    And I type "OAuth2"
    And I press Ctrl+S
    Then the screen should show "[3] Target platform?"
    And the detail panel should show "[3] Target platform?"

  Scenario: Tab is blocked during answer dialog
    When I switch to the questions tab
    And I press Enter
    And I press Tab
    Then the screen should show "Answer: Auth requirements?"
