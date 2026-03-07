Feature: Question Generation
  As a user writing requirements
  I want the integrator to surface clarifying questions with stable identifiers
  So that I can track and return to questions I'm considering

  Background:
    Given a clean working directory
    And the specwriter is running with a mock command

  Scenario: Integration produces questions with unique identifiers
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Q1."
    And the screen should not show "No open questions"

  Scenario: Questions retain identifiers across integrations
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Q1."
    And the screen should show "Q2."
    And the screen should show "Q3."
    When I type "Add a dashboard"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Q1."
    And the screen should show "Q2."
    And the screen should show "Q3."

  Scenario: New questions get identifiers continuing from the highest existing
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Q1."
    And the screen should show "Q2."
    And the screen should show "Q3."
    When I type "Users need to search for products"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Q4."
    And the screen should not show "Q3."

  Scenario: Answered questions are removed while unrelated questions persist
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Q1."
    And the screen should show "Q2."
    And the screen should show "Q3."
    When I type "We will use OAuth2 for authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should not show "Q1."
    And the screen should show "Q2."
    And the screen should show "Q3."
    And the screen should show "Q4."

  Scenario: Open questions are provided as context to the integrator
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Q1."
    When I type "Add a dashboard"
    And I press Ctrl+S
    And I wait for integration to complete
    Then SPEC.md should contain "questions-aware"

  Scenario: No questions when integrator output has none
    Given the specwriter is running with a no-questions mock
    When I type "Simple requirement"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "No open questions"

  Scenario: Pool unchanged when input produces no new questions
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Q1."
    And the screen should show "Q2."
    And the screen should show "Q3."
    When I type "Add a dashboard"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Q1."
    And the screen should show "Q2."
    And the screen should show "Q3."
    And the screen should show "What are the authentication requirements?"
