Feature: Screen Layout
  As a user
  I want the UI to have a clean, connected layout
  So that I can easily distinguish between status, tabs, content, and help

  Background:
    Given a clean working directory
    And the specwriter is running with a mock command

  Scenario: Status line is on row 1
    Then row 1 should start with the status icon followed by "Idle."

  Scenario: Row 2 is blank
    Then row 2 should be blank

  Scenario: Tab labels on row 3 have no borders
    Then row 3 should start with "  Text Input  Open Questions (0)"
    And row 3 should contain no box-drawing characters
    And the active tab title should be bold

  Scenario: Content border on row 4
    Then row 4 should start with a corner box-drawing character followed by horizontal lines

  Scenario: Content begins on row 5 with placeholder
    Then row 5 should start with a vertical border then "Type your requirements here. Ctrl+S to submit."
