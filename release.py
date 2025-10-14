#!/usr/bin/env python3
"""
Release script for csv-tools monorepo.

Reads version information from Cargo.toml files and creates releases
by tagging commits and triggering GitHub Actions workflows.
"""

import subprocess
import sys
from pathlib import Path

try:
    import tomllib
except ImportError:
    try:
        import tomli as tomllib
    except ImportError:
        print("Error: tomli library required for Python < 3.11")
        print("Install with: pip install tomli")
        sys.exit(1)


TOOLS = {
    "catcsv": "catcsv/Cargo.toml",
    "fixed2csv": "fixed2csv/Cargo.toml",
    "geochunk": "geochunk/Cargo.toml",
    "hashcsv": "hashcsv/Cargo.toml",
    "scrubcsv": "scrubcsv/Cargo.toml",
}


def run_command(cmd, check=True):
    """Run a command and return the result."""
    result = subprocess.run(cmd, capture_output=True, text=True)
    if check and result.returncode != 0:
        print(f"Error running command: {' '.join(cmd)}")
        print(f"stderr: {result.stderr}")
        sys.exit(1)
    return result.stdout.strip()


def get_version_from_cargo_toml(cargo_toml_path):
    """Extract version from a Cargo.toml file."""
    with open(cargo_toml_path, "rb") as f:
        data = tomllib.load(f)

    if "package" not in data or "version" not in data["package"]:
        raise ValueError(f"Could not find version in {cargo_toml_path}")

    return data["package"]["version"]


def get_current_versions():
    """Get current versions for all tools."""
    versions = {}
    for tool, cargo_toml in TOOLS.items():
        versions[tool] = get_version_from_cargo_toml(cargo_toml)
    return versions


def delete_existing_tags(versions):
    """Delete existing local and remote tags."""
    print("\nDeleting existing tags (if any)...")
    for tool, version in versions.items():
        tag = f"{tool}_v{version}"
        run_command(["git", "tag", "-d", tag], check=False)
        run_command(["git", "push", "origin", f":refs/tags/{tag}"], check=False)


def create_and_push_tags(versions, commit):
    """Create and push tags for all tools."""
    print(f"\nCreating tags on commit: {commit}")
    tags = []
    for tool, version in versions.items():
        tag = f"{tool}_v{version}"
        tags.append(tag)
        run_command(["git", "tag", tag, commit])
        print(f"  Created tag: {tag}")

    print("\nPushing tags...")
    run_command(["git", "push", "origin"] + tags)
    print("  Tags pushed successfully")


def trigger_workflows(versions):
    """Trigger GitHub Actions workflows for each tool."""
    print("\nTriggering release workflows...")
    for tool, version in versions.items():
        tag = f"{tool}_v{version}"
        workflow = f"ci-{tool}.yml"
        print(f"  Triggering {workflow} at {tag}...")
        result = run_command(["gh", "workflow", "run", workflow, "--ref", tag])
        if result:
            print(f"    {result}")


def main():
    print("=" * 60)
    print("CSV-Tools Release Script")
    print("=" * 60)

    # Get current versions
    versions = get_current_versions()

    print("\nCurrent versions from Cargo.toml files:")
    for tool, version in versions.items():
        print(f"  {tool:12s} v{version}")

    # Confirm
    print("\nThis will:")
    print("  1. Delete existing tags (if any)")
    print("  2. Create new tags on the current commit")
    print("  3. Push tags to origin")
    print("  4. Trigger GitHub Actions workflows to build and release")

    response = input("\nProceed with release? [y/N] ").strip().lower()
    if response not in ["y", "yes"]:
        print("Aborted.")
        sys.exit(0)

    # Get current commit
    commit = run_command(["git", "rev-parse", "HEAD"])

    # Delete existing tags
    delete_existing_tags(versions)

    # Create and push new tags
    create_and_push_tags(versions, commit)

    # Trigger workflows
    trigger_workflows(versions)

    print("\n" + "=" * 60)
    print("Done! Release workflows triggered.")
    print("=" * 60)
    print("\nCheck progress at:")
    print("  https://github.com/faradayio/csv-tools/actions")
    print("\nEach workflow will:")
    print("  1. Build binaries for all platforms")
    print("  2. Create a GitHub release")
    print("  3. Upload the binaries to the release")


if __name__ == "__main__":
    main()
