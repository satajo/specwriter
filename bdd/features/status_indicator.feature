Feature: Status Indicator
  As a user
  I want to see a visual indicator of the application state
  So that I can tell at a glance whether specwriter is idle, working, or has an error

  Background:
    Given a clean working directory
    And the specwriter is running with a mock command

  Scenario: Idle state shows no icon
    Then row 1 should start with " Idle."

  Scenario: Integrating state shows a dot spinner
    When I type "A requirement"
    And I press Ctrl+S
    Then the status line should contain yellow text

  Scenario: Error state shows red text
    Given the specwriter is running with a failing mock command
    When I type "Something"
    And I press Ctrl+S
    And I wait for integration to complete
    Then row 1 should start with " Error!"
    And the status line should contain red text

  Scenario: Integrating indicator spins
    When I type "A requirement"
    And I press Ctrl+S
    And time passes
    Then the status indicator should have animated
