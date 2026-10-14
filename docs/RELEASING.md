# Releases

The [release workflow](../.github/workflows/release.yml) builds locked release
binaries with Rust 1.89. Downloads contain only the application: a bare executable
on Linux/Windows or `EarthCraft.app` inside a macOS disk image.
These artifacts use `--no-default-features`: GPU rendering is excluded while
native cross-platform GPU failures remain unresolved.

| Platform | Artifact |
| --- | --- |
| Linux x86-64 | `earthcraft-0.1.0-linux-x64.tar.gz` |
| Linux ARM64 | `earthcraft-0.1.0-linux-arm64.tar.gz` |
| Windows x86-64 | `earthcraft-0.1.0-windows-x64-portable.zip` |
| Windows ARM64 | `earthcraft-0.1.0-windows-arm64-portable.zip` |
| macOS Intel and Apple Silicon | `earthcraft-0.1.0-macos-universal.dmg` containing `EarthCraft.app` |

One `SHA256SUMS.txt` lists all five release archives. Preview artifacts retain
individual `.sha256` files; publication verifies and consolidates them.
GitHub also provides its standard source-code archives. Linux builds run
on Ubuntu 22.04; GTK 3 and a working graphics driver are runtime requirements.
macOS bundles target macOS 11 or newer and are ad-hoc signed, without a
Developer ID certificate or notarization. Intel and Apple Silicon
executables build separately, then `lipo` combines them into the universal bundle.
Packaging verifies that both architectures are present, embeds the padded macOS
icon, and signs and verifies the combined app after assembly. Ad-hoc signing
does not establish a trusted publisher or remove Gatekeeper prompts.
The separate macOS build
artifacts are intermediate files and are not published as release downloads.
Windows/Linux archives contain just the executable at their root. On macOS,
open the DMG and copy `EarthCraft.app` to Applications. There are no bundled
READMEs, agent rules, contributor docs, changelogs or release text files. Build/package checks do not verify
interactive runtime behavior: manually test downloads on the supported platforms
before announcing them.

## Preview builds

Pull requests changing the release workflow, packaging code/tests, changelog
configuration, macOS icon or this guide run
six native architecture builds and upload preview artifacts without publishing
a release.
After merge, the Actions page also offers a manual **Release** run with the same
preview-only behavior. Inspect and test these artifacts before creating a version tag.
Actions artifact names also include the workspace version, so their downloadable
wrapper ZIPs are versioned (for example,
`earthcraft-0.1.0-x86_64-pc-windows-msvc.zip`). The release package inside retains
its friendly platform name from the table above.

## Publish a version

1. Set `workspace.package.version` in `Cargo.toml`, regenerate `Cargo.lock`,
   run the contributor checks, and merge those changes.
2. From the reviewed commit, create and push the matching tag, for example
   `git tag v0.1.0` followed by `git push upstream v0.1.0`.
3. The workflow requires the tag to equal `v` plus the workspace version.
   All six native builds, universal macOS packaging and archive checksums must
   succeed before publication.
4. Publication first creates a draft with GitHub-generated release notes, uploads the complete
   artifact set, then publishes it. Versions containing a prerelease suffix are
   marked as prereleases.

A failed upload leaves a draft that can be completed by rerunning the workflow.
A published release is not overwritten by a rerun; use a new version and tag.
Only the tag-triggered publication job has `contents: write`; build and preview
jobs have read-only repository access. No signing credentials or additional
personal access token are needed.

## Repository prerequisites

- Merge this workflow into the repository that will publish the release before
  running it manually or pushing a tag. Fork builds publish only to that fork.
- Enable GitHub Actions and permit the actions referenced in the workflow,
  including `dtolnay/rust-toolchain` and `orhun/git-cliff-action`.
  Repository/organization policy must allow the publication job's
  `contents: write` token permission.
- Ensure hosted ARM64 runners (`ubuntu-22.04-arm`, `windows-11-arm`) and the
  macOS runners in the matrix are available to the repository. Jobs queued
  indefinitely require checking runner availability, billing and Actions policy.
- Run the manual preview on the reviewed commit and verify all five archives,
  their checksums and actual application startup on each platform. A preview
  never publishes. No release tag is needed for it.
- Review generated release notes before tagging. Use a new matching version/tag;
  publication refuses to overwrite an already published release.

No crates.io publication, Apple signing secrets or Windows certificate are
required by this workflow. Trusted macOS distribution would additionally require
Developer ID signing and notarization, which this workflow does not configure.

## Generated changelog

The workflow runs git-cliff 2.14.2 through `orhun/git-cliff-action@v4` using
`cliff.toml` and full Git history. This is release tooling, not an application
dependency. It groups conventional commits and retains other commit subjects,
while skipping merge boilerplate. Only `vX.Y.Z`-style release tags define release
boundaries; legacy tags such as `2` and `versions` are ignored.

The `release-changelog` Actions artifact contains the generated `CHANGELOG.md`
with the full history; it is not included in application downloads. This is a history of changes,
so review it for features that were subsequently reverted. No generated file is
committed back to the repository, and no PR-label convention is required.

The GitHub release page uses GitHub's native generated notes rather than the
commit-grouped Actions changelog. This produces **What's Changed** entries with
PR titles, authors and PR links, a **New Contributors** section when applicable,
and a full comparison link when a previous comparison tag exists. The release title is
`EarthCraft v0.1.0`. Existing draft notes are preserved on a retry.

Maintainers can preview the GitHub notes without creating a release:

```sh
gh api --method POST repos/therichronicles/EarthCraft/releases/generate-notes -f tag_name=v0.1.0 -f target_commitish=main --jq .body
```

To preview locally, install git-cliff 2.14.2 and run:

```sh
git-cliff --config cliff.toml --output /tmp/earthcraft-CHANGELOG.md
python3 -m unittest discover -s scripts -p 'test_package_release.py'
```

## Local packaging

Python 3.11+ is required for the packaging helper. For a native Linux x86-64 build:

```sh
python3 scripts/package_release.py validate --tag v0.1.0
cargo build -p earthcraft --release --no-default-features --locked
python3 scripts/package_release.py package --version 0.1.0 --target x86_64-unknown-linux-gnu --binary target/release/earthcraft --output /tmp/earthcraft-release
```

Use the target and executable name for your native platform. For explicit-target
builds, the executable lives under `target/<target>/release/`.
The helper packages binaries; it does not cross-compile them. The universal macOS
packaging target additionally requires `lipo`, `codesign` and `hdiutil`. It verifies
both architecture slices, signs the app, creates a compressed read-only DMG and
verifies the disk image.

To reproduce the universal macOS artifact:

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
MACOSX_DEPLOYMENT_TARGET=11.0 cargo build -p earthcraft --release --no-default-features --locked --target aarch64-apple-darwin
MACOSX_DEPLOYMENT_TARGET=11.0 cargo build -p earthcraft --release --no-default-features --locked --target x86_64-apple-darwin
lipo -create target/aarch64-apple-darwin/release/earthcraft target/x86_64-apple-darwin/release/earthcraft -output /tmp/earthcraft-universal
python3 scripts/package_release.py package --version 0.1.0 --target universal-apple-darwin --binary /tmp/earthcraft-universal --output /tmp/earthcraft-release
```
