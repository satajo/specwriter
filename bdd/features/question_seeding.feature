Feature: Startup with Existing Spec
  As a user returning to a project with an existing spec
  I want specwriter to display questions already in the spec files
  So that I can pick up where I left off

  Background:
    Given a clean working directory

  Scenario: Starting with an existing spec displays its questions
    Given the spec README already contains "# My App\n\nA web application.\n\n## Questions\n\n### Q1 (p7): What is the primary user persona?\n\nDescribe the target user.\n\n### Q2 (p5): Are there performance requirements?\n\nAny latency or throughput targets?"
    And the specwriter is running with a mock command
    And I switch to the questions tab
    Then the screen should show "Q1 (p7)."
    And the screen should show "Q2 (p5)."
    And the screen should not show "No open questions"

  Scenario: Questions show which spec file they come from
    Given the spec README already contains "# My App\n\nA web application.\n\n## Questions\n\n### Q1 (p7): What is the primary user persona?\n\nDescribe the target user."
    And the specwriter is running with a mock command
    And I switch to the questions tab
    Then the screen should show "README.md"

  Scenario: Starting without an existing spec shows no questions
    Given the specwriter is running with a mock command
    Then the screen should show "Ready"
    And the screen should show "Open questions (0)"
