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
    Then the screen should not show "1/1"

  Scenario: Multiple submissions show queue progress
    Given the specwriter is running with a slow mock command
    When I type "search"
    And I press Ctrl+S
    And I type "filter"
    And I press Ctrl+S
    And I wait for status to contain "1/2"
    Then the screen should show "Integrating 1/2"
    When I wait for all integrations to finish
    Then SPEC.md should contain "search"
    And SPEC.md should contain "filter"
    And the screen should show "Ready"
