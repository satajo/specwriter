Feature: Question Generation
  As a user writing requirements
  I want the integrator to surface clarifying questions with stable identifiers
  So that I can track and return to questions I'm considering

  Background:
    Given a clean working directory
    And the specwriter is running with a mock command

  Scenario: Integration produces questions
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    Then the screen should show "[5] What are the authentication"
    And the screen should not show "No open questions"

  Scenario: Questions retain across integrations
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    Then the screen should show "[5] What are the authentication"
    And the screen should show "[3] Should there be role"
    And the screen should show "[2] What is the target"
    When I switch to the text input tab
    And I type "Add a dashboard"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    Then the screen should show "[5] What are the authentication"
    And the screen should show "[3] Should there be role"
    And the screen should show "[2] What is the target"

  Scenario: New questions appear when old ones are removed
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    Then the screen should show "[5] What are the authentication"
    And the screen should show "[3] Should there be role"
    And the screen should show "[2] What is the target"
    When I switch to the text input tab
    And I type "Users need to search for products"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    Then the screen should show "[3] What search fields"
    And the screen should not show "[2] What is the target"

  Scenario: Answered questions are removed while unrelated questions persist
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    Then the screen should show "[5] What are the authentication"
    And the screen should show "[3] Should there be role"
    And the screen should show "[2] What is the target"
    When I switch to the text input tab
    And I type "We will use OAuth2 for authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    Then the screen should not show "authentication requirements"
    And the screen should show "[3] Should there be role"
    And the screen should show "[2] What is the target"
    And the screen should show "[4] What OAuth providers"

  Scenario: Questions are placed under a Questions heading in spec files
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    Then the spec should contain "## Questions"
    And the spec should contain "Q1"
    And the spec should contain "Q2"
    And the spec should contain "Q3"

  Scenario: No questions when integrator output has none
    Given the specwriter is running with a no-questions mock
    When I type "Simple requirement"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    Then the screen should show "No open questions"

  Scenario: Focus preserved by question ID after integration refresh
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    And I press Down
    Then the detail panel should show "[3] Should there be role-based"
    When I switch to the text input tab
    And I type "Add a dashboard"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    Then the detail panel should show "[3] Should there be role-based"

  Scenario: Focus falls back after focused question is removed by integration
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    And I press Down
    And I press Down
    Then the detail panel should show "[2] What is the target"
    When I switch to the text input tab
    And I type "Users need to search for products"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    Then the screen should not show "What is the target"
    And the detail panel should show "[3] What search fields"

  Scenario: Pool unchanged when input produces no new questions
    When I type "The app needs user authentication"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    Then the screen should show "[5] What are the authentication"
    And the screen should show "[3] Should there be role"
    And the screen should show "[2] What is the target"
    When I switch to the text input tab
    And I type "Add a dashboard"
    And I press Ctrl+S
    And I wait for integration to complete
    And I switch to the questions tab
    Then the screen should show "[5] What are the authentication"
    And the screen should show "[3] Should there be role"
    And the screen should show "[2] What is the target"
    And the screen should show "What are the authentication requirements?"
