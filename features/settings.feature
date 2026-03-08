Feature: Configurable settings
  As a user
  I want to configure how specwriter invokes the claude CLI
  So that the tool works across different environments and setups

  Background:
    Given a clean working directory
    And the specwriter is running with a mock command

  Scenario: Settings tab is visible in the tab bar
    Then the screen should show "Settings"

  Scenario: Settings tab shows default values
    When I switch to the settings tab
    Then the screen should show "Claude Command"
    And the screen should show "claude"
    And the screen should show "Model"
    And the screen should show "(not set)"
    And the screen should show "Spec Filename"
    And the screen should show "SPEC.md"
    And the screen should show "WebSearch"
    And the screen should show "WebFetch"
    And the screen should show "off"

  Scenario: Down arrow navigates settings list
    When I switch to the settings tab
    And I press Down
    And I press Down
    Then the screen should show "Spec Filename"

  Scenario: Up arrow navigates settings list
    When I switch to the settings tab
    And I press Down
    And I press Down
    And I press Up
    Then the screen should show "Model"

  # --- Enter to confirm, Esc to cancel ---

  Scenario: Enter opens inline editor for text field
    When I switch to the settings tab
    And I press Enter
    And I type "myclaude"
    Then the screen should show "myclaude"

  Scenario: Enter confirms text edit
    When I switch to the settings tab
    And I press Enter
    And I type "/usr/bin/claude"
    And I press Enter
    Then the screen should show "/usr/bin/claude"
    And the screen should not show "type to edit"

  Scenario: Esc cancels text edit and reverts value
    When I switch to the settings tab
    And I press Enter
    And I type "/usr/bin/claude"
    And I press Esc
    Then the screen should not show "/usr/bin/claude"
    And the screen should show "claude"

  Scenario: Enter toggles boolean setting on
    When I switch to the settings tab
    And I press Down
    And I press Down
    And I press Down
    And I press Enter
    Then the screen should show "on"

  Scenario: Toggling boolean twice returns to off
    When I switch to the settings tab
    And I press Down
    And I press Down
    And I press Down
    And I press Enter
    And I press Enter
    Then the screen should not show "WebSearch         on"

  Scenario: Model shows not set when cleared
    When I switch to the settings tab
    And I press Down
    And I press Enter
    And I press Enter
    Then the screen should show "(not set)"

  # --- Help text ---

  Scenario: Help text shows settings navigation keys
    When I switch to the settings tab
    Then the screen should show "Enter: edit/toggle"
    And the screen should show "Ctrl+S: save"

  Scenario: Help text changes during editing
    When I switch to the settings tab
    And I press Enter
    Then the screen should show "Enter: confirm"
    And the screen should show "Esc: cancel"

  Scenario: Esc is not shown in non-editing help text
    When I switch to the settings tab
    Then the screen should not show "Esc:"

  # --- Explicit Ctrl+S save with dialog ---

  Scenario: Ctrl+S shows save confirmation dialog
    When I switch to the settings tab
    And I press Enter
    And I type "/opt/claude"
    And I press Enter
    And I press Ctrl+S
    Then the screen should show "Save settings?"
    And the screen should show "Restart"

  Scenario: Enter in save dialog saves to disk and dismisses
    When I switch to the settings tab
    And I press Enter
    And I type "/opt/claude"
    And I press Enter
    And I press Ctrl+S
    And I press Enter
    Then the settings file should contain "/opt/claude"
    And the screen should not show "Save settings?"

  Scenario: Esc in save dialog cancels without saving
    When I switch to the settings tab
    And I press Enter
    And I type "/opt/claude"
    And I press Enter
    And I press Ctrl+S
    And I press Esc
    Then the settings file should not exist
    And the screen should not show "Save settings?"
    And the screen should show "/opt/claude"

  Scenario: Edits are not saved to disk without Ctrl+S
    When I switch to the settings tab
    And I press Enter
    And I type "/opt/claude"
    And I press Enter
    Then the settings file should not exist

  # --- Settings persist across tab switches ---

  Scenario: Unsaved edits are preserved when switching tabs
    When I switch to the settings tab
    And I press Enter
    And I type "/opt/claude"
    And I press Enter
    When I press Tab
    And I switch to the settings tab
    Then the screen should show "/opt/claude"

  # --- Loading from file ---

  Scenario: Settings are loaded from file on startup
    Given a settings file with claude command "/custom/claude"
    And the specwriter is running with a mock command
    When I switch to the settings tab
    Then the screen should show "/custom/claude"

  Scenario: Malformed settings file shows error and uses defaults
    Given a settings file with invalid content
    And the specwriter is running with a mock command
    Then the screen should show "Error"
    When I switch to the settings tab
    Then the screen should show "claude"

  # --- Boolean toggle saves are also deferred ---

  Scenario: Boolean toggle is not saved to disk without Ctrl+S
    When I switch to the settings tab
    And I press Down
    And I press Down
    And I press Down
    And I press Enter
    Then the screen should show "on"
    And the settings file should not exist

  Scenario: Boolean toggle is saved after Ctrl+S confirm
    When I switch to the settings tab
    And I press Down
    And I press Down
    And I press Down
    And I press Enter
    And I press Ctrl+S
    And I press Enter
    Then the settings file should contain "true"
