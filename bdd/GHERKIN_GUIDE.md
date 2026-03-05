# Writing good Gherkin

Reference guide for writing BDD feature files in this project.

## Scenarios describe behavior, not mechanics

Scenarios are written from the user's perspective. They say *what* the user does and *what* they observe — never how the system works internally.

- Good: `When I type "login page needed"` / `Then the screen should show "Integration complete"`
- Bad: `When the UnboundedSender receives a String` / `Then the Vec<String> should have length 3`

The only exception is verifying side effects that ARE the product (e.g., `Then SPEC.md should exist` — the file is what the user cares about).

## Declarative over imperative

Steps express intent, not low-level mechanics. Combine atomic actions into meaningful steps.

- Good: `When I type "hello"` (maps to user intent)
- Bad: `When I press 'h'` / `When I press 'e'` / `When I press 'l'` / ...

For a TUI, keybindings like `When I press Ctrl+S` are appropriate because the keybinding IS the user's interaction model.

## One behavior per scenario

Each scenario tests a single behavior. Multiple `Then` assertions are fine if they verify facets of the same outcome. If you're writing "and then also check this unrelated thing" — split it.

## Scenarios must be independent

No scenario depends on another having run. Each starts clean (use `Background` for shared setup). This means:
- No shared mutable state between scenarios
- `Background` sets up preconditions common to all scenarios in a feature
- Keep backgrounds short — 2-3 Given steps max

## Feature descriptions guide scenario design

The `As a / I want / So that` header isn't decoration. It establishes who benefits and why. If you can't articulate the "so that", the feature may not need to exist. Scenarios should trace back to the value described in the header.

## Use domain language

Steps use the vocabulary of the user, not the codebase. The feature file should be understandable by someone who has never read the Rust code.

- Good: `Then the screen should show "Error"`
- Bad: `Then the status field should match regex "Error.*"`

## Keep scenarios short

3-7 steps is the sweet spot. If a scenario exceeds ~10 steps, it's testing too much or the steps are too granular. Refactor into either multiple scenarios or higher-level steps.

## Scenario names tell the story

The name should convey what behavior is verified without reading the steps.

- Good: `Scenario: Input is cleared after submission`
- Bad: `Scenario: Test case 4`

## Step definitions are thin adapters

Step definition code should be a minimal bridge between Gherkin and the system under test. No complex logic, no branching, no conditionals on parameters. If a step needs an if/else, write two different steps instead.

## Reuse steps, but not at the cost of clarity

Generic parameterized steps like `the screen should show {string}` are good. But don't over-abstract into steps so generic they obscure meaning. Clarity wins over DRY.

## Use Scenario Outline for data-driven variations

When the same behavior applies to multiple inputs, use `Scenario Outline` with `Examples` tables rather than copy-pasting scenarios that differ only in data.

## Don't test the framework

Don't write scenarios verifying that ratatui renders borders correctly or that tokio channels deliver messages. Test YOUR behavior — the user-visible outcomes of the product.
