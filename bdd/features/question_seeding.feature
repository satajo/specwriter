Feature: Startup with Existing Spec
  As a user returning to a project with an existing spec
  I want specwriter to display questions already in the spec files
  So that I can pick up where I left off

  Background:
    Given a clean working directory

  Scenario: Starting with an existing spec displays its questions
    Given the spec README already contains "# My App\n\nA web application.\n\n## Questions\n\nQ1: What is the primary user persona?\n\nQ2: Are there performance requirements?"
    And the specwriter is running with a mock command
    Then the screen should show "Q1."
    And the screen should show "Q2."
    And the screen should not show "No open questions"

  Scenario: Questions show which spec file they come from
    Given the spec README already contains "# My App\n\nA web application.\n\n## Questions\n\nQ1: What is the primary user persona?"
    And the specwriter is running with a mock command
    Then the screen should show "README.md"

  Scenario: Starting without an existing spec shows no questions
    Given the specwriter is running with a mock command
    Then the screen should show "Ready"
    And the screen should show "No open questions"
