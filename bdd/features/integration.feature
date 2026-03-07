Feature: Spec Integration
  As a user writing requirements
  I want my messages to be integrated into a cohesive SPEC.md
  So that I have a single source of truth for my project's requirements

  Background:
    Given a clean working directory
    And the specwriter is running with a mock command

  Scenario: Application starts with a ready screen
    Then the screen should show "Ready"
    And the screen should show "Open Questions"
    And the screen should show "Input"
    And the screen should show "Ctrl+S"

  Scenario: SPEC.md does not exist at launch
    Then SPEC.md should not exist

  Scenario: First message creates a new spec
    When I type "The app should have a login page"
    And I press Ctrl+S
    And I wait for integration to complete
    Then SPEC.md should exist
    And SPEC.md should contain "login"
    And the screen should show "Ready"

  Scenario: Subsequent messages update the existing spec
    Given SPEC.md already contains "# Spec\n\nThe app has a login page."
    When I type "Users should also be able to reset their password"
    And I press Ctrl+S
    And I wait for integration to complete
    Then SPEC.md should contain "password"

  Scenario: Submitting empty text does nothing
    When I type "   "
    And I press Ctrl+S
    Then the screen should show "Ready"

  Scenario: Input is cleared after submission
    When I type "Some requirement"
    Then the input area should show "Some requirement"
    When I press Ctrl+S
    Then the input area should not show "Some requirement"
    And the screen should show "Integrating"

  Scenario: Rapid submissions are all integrated
    When I type "Feature A: search functionality"
    And I press Ctrl+S
    And I type "Feature B: filtering results"
    And I press Ctrl+S
    And I wait for all integrations to finish
    Then SPEC.md should contain "search"
    And SPEC.md should contain "filtering"
    And the screen should show "Ready"
