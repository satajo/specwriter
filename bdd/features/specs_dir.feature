Feature: Custom Specs Directory
  As a user
  I want to configure the specs directory name
  So that I can organize my project files as needed

  Background:
    Given a clean working directory

  Scenario: Custom specs directory reads questions on startup
    Given specs are in directory "custom_specs" with content "# App\n\n## Questions\n\n### Q1 (p5): Custom dir test?\n\nBody."
    And the specwriter is running with specs dir "custom_specs" and a mock command
    And I switch to the questions tab
    Then the screen should show "Q1 (p5)."

  Scenario: Default specs directory is used when not overridden
    Given the spec README already contains "# App\n\n## Questions\n\n### Q1 (p5): Default dir test?\n\nBody."
    And the specwriter is running with a mock command
    And I switch to the questions tab
    Then the screen should show "Q1 (p5)."
