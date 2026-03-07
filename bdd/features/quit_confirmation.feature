Feature: Quit Confirmation
  As a user
  I want to be warned before quitting during an integration
  So that I don't accidentally lose an in-progress integration

  Background:
    Given a clean working directory

  Scenario: Ctrl+C while idle quits immediately
    Given the specwriter is running with a mock command
    When I press Ctrl+C
    Then the app should have quit

  Scenario: Ctrl+C during integration shows confirmation dialog
    Given the specwriter is running with a slow mock command
    When I type "test requirement"
    And I press Ctrl+S
    And I press Ctrl+C
    Then the app should not have quit
    And the screen should show "Confirm Quit"
    And the screen should show "Press Ctrl+C again to quit"

  Scenario: Second Ctrl+C during integration quits
    Given the specwriter is running with a slow mock command
    When I type "test requirement"
    And I press Ctrl+S
    And I press Ctrl+C
    And I press Ctrl+C
    Then the app should have quit

  Scenario: Esc dismisses quit confirmation dialog
    Given the specwriter is running with a slow mock command
    When I type "test requirement"
    And I press Ctrl+S
    And I press Ctrl+C
    Then the screen should show "Confirm Quit"
    When I press Esc
    Then the screen should not show "Confirm Quit"
    And the app should not have quit

  Scenario: Tab is blocked during quit confirmation
    Given the specwriter is running with a slow mock command
    When I type "test requirement"
    And I press Ctrl+S
    And I press Ctrl+C
    Then the screen should show "Confirm Quit"
    When I press Tab
    Then the screen should show "Confirm Quit"

  Scenario: Help bar shows quit confirmation shortcuts
    Given the specwriter is running with a slow mock command
    When I type "test requirement"
    And I press Ctrl+S
    And I press Ctrl+C
    Then the screen should show "Esc: cancel"
