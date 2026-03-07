Feature: Input Placeholder
  As a user
  I want to see helpful placeholder text in the empty input area
  So that I know what to do when I first start the app

  Background:
    Given a clean working directory
    And the specwriter is running with a mock command

  Scenario: Empty text input shows placeholder text
    Then the screen should show "Ctrl+S to submit"

  Scenario: Placeholder disappears when user types
    When I type "hello"
    Then the screen should not show "Ctrl+S to submit"
    And the input area should show "hello"
