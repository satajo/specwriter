Feature: Spec Integration
  As a user writing requirements
  I want my messages to be integrated into a cohesive SPEC.md
  So that I have a single source of truth for my project's requirements

  Background:
    Given a clean working directory
    And the integrator is configured with a mock command

  Scenario: First message creates a new spec
    When I submit the message "The app should have a login page"
    And I wait for integration to complete
    Then SPEC.md should exist
    And SPEC.md should contain "login"

  Scenario: Subsequent messages update the existing spec
    Given SPEC.md already contains "# Spec\n\nThe app has a login page."
    When I submit the message "Users should also be able to reset their password"
    And I wait for integration to complete
    Then SPEC.md should contain "password"

  Scenario: Submitting empty text does nothing
    When I type "   "
    And I submit
    Then the status should be "Ready. Type your requirements and press Ctrl+S to submit."
    And no integration should have been triggered

  Scenario: Multiple rapid submissions are batched
    When I submit the message "Feature A: search functionality"
    And I immediately submit the message "Feature B: filtering results"
    And I wait for integration to complete
    Then the integrator should have received both messages in one batch
