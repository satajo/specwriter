Feature: Queue Progress
  As a user submitting multiple requirements
  I want to see progress through the queue
  So that I know how many messages are left to process

  Background:
    Given a clean working directory

  Scenario: Single submission shows no queue info
    Given the specwriter is running with a mock command
    When I type "A simple requirement"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the screen should not show "in queue"

  Scenario: Multiple submissions show queue count
    Given the specwriter is running with a slow mock command
    When I type "search"
    And I press Ctrl+S
    And I type "filter"
    And I press Ctrl+S
    And I wait for status to contain "in queue"
    Then the screen should show "Integrating (1 in queue)"
    When I wait for all integrations to finish
    Then the spec should contain "search"
    And the spec should contain "filtering"
    And the screen should show "Idle."

  Scenario: Queue count updates immediately on new submission
    Given the specwriter is running with a slow mock command
    When I type "first"
    And I press Ctrl+S
    And I type "second"
    And I press Ctrl+S
    And I wait for status to contain "in queue"
    When I type "third"
    And I press Ctrl+S
    And I wait for status to contain "2 in queue"
    Then the screen should show "2 in queue"
    When I wait for all integrations to finish
    Then the screen should show "Idle."
