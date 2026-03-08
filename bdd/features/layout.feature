Feature: Screen Layout
  As a user
  I want the UI to have a clean, connected layout
  So that I can easily distinguish between status, tabs, content, and help

  Background:
    Given a clean working directory
    And the specwriter is running with a mock command

  Scenario: Status line is on row 1
    Then row 1 should start with " Idle."

  Scenario: Row 2 is blank
    Then row 2 should be blank

  Scenario: Tab labels on row 3 have no borders
    Then row 3 should start with " Text Input  Open Questions (0)"
    And row 3 should contain no box-drawing characters
    And the active tab title should be bold

  Scenario: Text input tab has bordered content on row 4
    Then row 4 should start with a corner box-drawing character followed by horizontal lines

  Scenario: Content begins on row 5 with placeholder
    Then row 5 should start with a vertical border then "Type your requirements here. Ctrl+S to submit."

  Scenario: Questions tab with no questions shows single bordered list box
    Given the specwriter is running with a mock command
    When I switch to the questions tab
    Then row 4 should start with a corner box-drawing character followed by horizontal lines
    And the screen should show "No open questions"

  Scenario: Questions tab with questions shows two separate bordered boxes
    Given SPEC.md already contains "# App\n\n## Questions\n\n### Q1 (p8): Auth?\n\nHow?\n\n### Q2 (p5): Platform?\n\nWhich?"
    And the specwriter is running with a mock command
    When I switch to the questions tab
    Then the screen should show "Q1 (p8)"
    And the screen should show "Q2 (p5)"
