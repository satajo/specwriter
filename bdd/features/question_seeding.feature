Feature: Question Seeding from Existing Spec
  As a user returning to a project with an existing spec
  I want specwriter to generate initial questions from the existing spec
  So that I can pick up where I left off

  Background:
    Given a clean working directory

  Scenario: Starting with an existing spec generates initial questions
    Given the spec README already contains "# My App\n\nA web application for managing tasks."
    And the specwriter is running with a seeding mock
    And I wait for seeding to complete
    Then the screen should show "Q1."
    And the screen should not show "No open questions"

  Scenario: Startup shows loading status while seeding
    Given the spec README already contains "# My App\n\nA web application for managing tasks."
    And the specwriter is running with a slow seeding mock
    When I wait for status to contain "Loading"
    Then the screen should show "Loading existing specs"

  Scenario: Starting without an existing spec does not seed
    Given the specwriter is running with a seeding mock
    Then the screen should show "Ready"
    And the screen should show "No open questions"
