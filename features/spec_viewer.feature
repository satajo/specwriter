Feature: Spec viewer tab
  As a user
  I want to view the current spec content inside the TUI
  So that I can read SPEC.md without leaving the app

  Background:
    Given a clean working directory
    And the specwriter is running with a mock command

  Scenario: Spec tab always shows filename with line count
    Then the screen should show "SPEC.md (0)"

  Scenario: Tab cycles through all three tabs
    When I press Tab
    Then the screen should show "navigate"
    When I press Tab
    Then the screen should show "No spec file yet"
    When I press Tab
    Then the screen should show "Ctrl+S"

  Scenario: Spec tab shows line count when file exists
    Given SPEC.md already contains "# Title\n\nSome content\nMore content"
    And the specwriter is running with a mock command
    Then the screen should show "SPEC.md (4)"

  Scenario: Spec tab shows zero line count for empty file
    Given SPEC.md already contains ""
    And the specwriter is running with a mock command
    Then the screen should show "SPEC.md (0)"

  Scenario: Missing spec file shows placeholder message
    When I switch to the spec tab
    Then the screen should show "No spec file yet"

  Scenario: Spec tab displays raw file content
    Given SPEC.md already contains "# My Spec\n\nThis is a requirement."
    And the specwriter is running with a mock command
    When I switch to the spec tab
    Then the screen should show "# My Spec"
    And the screen should show "This is a requirement."

  Scenario: Spec tab scrolls with arrow keys
    Given SPEC.md already contains "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12\nLine 13\nLine 14\nLine 15\nLine 16\nLine 17\nLine 18\nLine 19\nLine 20\nLine 21\nLine 22\nLine 23\nLine 24\nLine 25\nLine 26\nLine 27\nLine 28\nLine 29\nLine 30"
    And the specwriter is running with a mock command
    When I switch to the spec tab
    And I press Down 25 times
    Then the screen should show "Line 30"

  Scenario: Scroll position is preserved across tab switches
    Given SPEC.md already contains "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12\nLine 13\nLine 14\nLine 15\nLine 16\nLine 17\nLine 18\nLine 19\nLine 20\nLine 21\nLine 22\nLine 23\nLine 24\nLine 25\nLine 26\nLine 27\nLine 28\nLine 29\nLine 30"
    And the specwriter is running with a mock command
    When I switch to the spec tab
    And I press Down 25 times
    And I press Tab
    And I switch to the spec tab
    Then the screen should show "Line 30"

  Scenario: Spec tab shows correct help line
    When I switch to the spec tab
    Then the screen should show "scroll"
    And the screen should show "Ctrl+C"
    And the screen should show "Tab"

  Scenario: Spec content refreshes after integration
    When I type "add a login page"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should show "SPEC.md ("
