Feature: Nix Packaging
  As a user installing specwriter
  I want the nix flake to produce a working package
  So that I can install and run it via nix

  Scenario: Nix flake builds successfully
    Given a clean working directory
    Then "nix build" should succeed
    And the nix build output should contain a "specwriter" binary
