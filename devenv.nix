{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  languages.rust.enable = true;

  git-hooks.hooks = {
    rustfmt = {
      enable = true;
      entry = "${pkgs.cargo}/bin/cargo fmt --all";
    };

    clippy = {
      enable = true;
      entry = "${pkgs.cargo}/bin/cargo clippy --workspace --all-features -- -D warnings";
      pass_filenames = false;
    };

    tests = {
      enable = true;
      entry = "${pkgs.cargo}/bin/cargo test --workspace";
      pass_filenames = false;
      stages = ["pre-push"];
    };
  };

  scripts = {
    release = {
      description = ''bump version, create tag, and push'';
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail

        # Check if version argument is provided
        if [ $# -eq 0 ]; then
          echo "Usage: release <major|minor|patch|VERSION>"
          echo "Examples:"
          echo "  release patch    # 0.1.0 -> 0.1.1"
          echo "  release minor    # 0.1.0 -> 0.2.0"
          echo "  release major    # 0.1.0 -> 1.0.0"
          echo "  release 0.2.3    # Set specific version"
          exit 1
        fi

        # Get current version from workspace
        CURRENT_VERSION=$(grep '^\[workspace.package\]' -A 10 Cargo.toml | grep '^version' | head -1 | sed 's/.*"\(.*\)".*/\1/')
        echo "Current version: $CURRENT_VERSION"

        # Calculate new version
        if [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
          NEW_VERSION="$1"
        else
          IFS='.' read -r major minor patch <<< "$CURRENT_VERSION"
          case "$1" in
            major)
              NEW_VERSION="$((major + 1)).0.0"
              ;;
            minor)
              NEW_VERSION="$major.$((minor + 1)).0"
              ;;
            patch)
              NEW_VERSION="$major.$minor.$((patch + 1))"
              ;;
            *)
              echo "Error: Invalid version bump type or version number"
              exit 1
              ;;
          esac
        fi

        echo "New version: $NEW_VERSION"
        read -p "Proceed with release? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
          echo "Release cancelled"
          exit 1
        fi

        # Check working directory is clean
        if [ -n "$(git status --porcelain)" ]; then
          echo "Error: Working directory is not clean. Commit or stash changes first."
          exit 1
        fi

        # Update version in workspace Cargo.toml
        echo "Updating version in Cargo.toml..."
        sed -i.bak "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
        rm Cargo.toml.bak

        # Update Cargo.lock
        echo "Updating Cargo.lock..."
        cargo check --workspace

        # Commit version bump
        echo "Committing version bump..."
        git add Cargo.toml Cargo.lock
        git commit -m "chore: bump version to v$NEW_VERSION"

        # Create and push tag
        echo "Creating tag v$NEW_VERSION..."
        git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"

        echo ""
        echo "✅ Release prepared!"
        echo ""
        echo "Next steps:"
        echo "  1. Review the commit and tag"
        echo "  2. Push to trigger release: git push && git push --tags"
        echo ""
        echo "This will trigger GitHub Actions to:"
        echo "  - Run all tests and checks"
        echo "  - Publish protest-derive to crates.io"
        echo "  - Publish protest to crates.io"
        echo "  - Create a GitHub release"
      '';
    };

    release-dry = {
      description = ''Dry-run release (shows what would happen without making changes)'';
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail

        if [ $# -eq 0 ]; then
          echo "Usage: release-dry <major|minor|patch|VERSION>"
          exit 1
        fi

        CURRENT_VERSION=$(grep '^\[workspace.package\]' -A 10 Cargo.toml | grep '^version' | head -1 | sed 's/.*"\(.*\)".*/\1/')
        echo "Current version: $CURRENT_VERSION"

        if [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
          NEW_VERSION="$1"
        else
          IFS='.' read -r major minor patch <<< "$CURRENT_VERSION"
          case "$1" in
            major) NEW_VERSION="$((major + 1)).0.0" ;;
            minor) NEW_VERSION="$major.$((minor + 1)).0" ;;
            patch) NEW_VERSION="$major.$minor.$((patch + 1))" ;;
            *)
              echo "Error: Invalid version bump type"
              exit 1
              ;;
          esac
        fi

        echo "Would bump to: $NEW_VERSION"
        echo ""
        echo "Changes that would be made:"
        echo "  1. Update Cargo.toml version"
        echo "  2. Update Cargo.lock"
        echo "  3. Run tests and clippy"
        echo "  4. Commit: 'chore: bump version to v$NEW_VERSION'"
        echo "  5. Create tag: v$NEW_VERSION"
        echo ""
        echo "Run 'release $1' to actually perform the release"
      '';
    };
  };
}
