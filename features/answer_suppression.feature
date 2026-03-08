Feature: Answered question suppression
  As a user who answers a question during an ongoing integration
  I want the answered question to stay dismissed
  So that concurrent integration results don't undo my answer

  Background:
    Given a clean working directory
    Given SPEC.md already contains "# App\n\n## Questions\n\n### Q1 (p5): Auth requirements?\n\nHow should users authenticate?\n\n### Q2 (p3): Target platform?\n\nWeb, mobile, or desktop?"
    And the specwriter is running with a slow mock command

  Scenario: Answered question does not reappear when integration refreshes questions
    When I switch to the questions tab
    And I press Enter
    And I type "OAuth2"
    And I press Ctrl+S
    Then the screen should not show "Auth requirements?"
    When the integrator reports questions from the spec file
    Then the screen should not show "Auth requirements?"
    And the screen should show "[3] Target platform?"
