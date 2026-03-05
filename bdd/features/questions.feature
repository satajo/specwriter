Feature: Question Generation
  As a user writing requirements
  I want the integrator to surface clarifying questions
  So that I can think about gaps in my specification

  Background:
    Given a clean working directory
    And the specwriter is running with a mock command

  Scenario: Integration produces questions on screen
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "1."
    And the screen should not show "No open questions"

  Scenario: Questions are refreshed on subsequent integrations
    When I type "Users can create projects"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "1."
    When I type "Projects have a name and description"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "1."
    And the screen should not show "No open questions"

  Scenario: No questions when integrator output has none
    Given the specwriter is running with a no-questions mock
    When I type "Simple requirement"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "No open questions"
