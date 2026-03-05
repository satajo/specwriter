Feature: Question Generation
  As a user writing requirements
  I want the integrator to surface clarifying questions
  So that I can think about gaps in my specification

  Background:
    Given a clean working directory
    And the integrator is configured with a mock command

  Scenario: Integration produces questions
    When I submit the message "The app needs user authentication"
    And I wait for integration to complete
    Then I should see questions displayed
    And there should be at most 3 questions

  Scenario: Questions update after each integration
    When I submit the message "Users can create projects"
    And I wait for integration to complete
    And I submit the message "Projects have a name and description"
    And I wait for integration to complete
    Then the questions should have been updated

  Scenario: No questions when integrator output has none
    Given the mock command will not produce questions
    When I submit the message "Simple requirement"
    And I wait for integration to complete
    Then I should see no questions displayed
