Feature: Error Handling
  As a user
  I want clear feedback when integration fails
  So that I know something went wrong and can take action

  Background:
    Given a clean working directory

  Scenario: Integrator command not found
    Given the specwriter is running with command "nonexistent-command-xyz"
    When I type "Some requirement"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Error!"

  Scenario: Integrator command exits with error
    Given the specwriter is running with a failing mock command
    When I type "Some requirement"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Error!"

  Scenario: Submitting new input recovers from error
    Given the specwriter is running with a failing mock command
    When I type "Something"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "Error!"
    When I type "Try again"
    And I press Ctrl+S
    Then the screen should show "Integrating"
