Feature: Error Handling
  As a user
  I want clear feedback when integration fails
  So that I know something went wrong and can take action

  Background:
    Given a clean working directory

  Scenario: Integrator command not found
    Given the integrator is configured with command "nonexistent-command-xyz"
    When I submit the message "Some requirement"
    And I wait for integration to complete
    Then the status should contain "Error"

  Scenario: Integrator command exits with error
    Given the integrator is configured with a failing mock command
    When I submit the message "Some requirement"
    And I wait for integration to complete
    Then the status should contain "Error"
