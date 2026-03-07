Feature: Session Recovery
  As a user
  I want the integrator to recover from session errors transparently
  So that I don't need to restart the application

  Background:
    Given a clean working directory

  Scenario: Integrator recovers from a session error by creating a new session
    Given the specwriter is running with a session-expiry mock
    When I type "test requirement"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the spec README should exist
    And the screen should show "Idle."
