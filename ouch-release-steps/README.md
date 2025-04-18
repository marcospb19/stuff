1. Edit start of CHANGELOG, replacing
  ```md
  - ## [Unreleased](https://github.com/ouch-org/ouch/compare/PREVIOUS_VERSION...HEAD)
  ```
  by
  ```md
  ## [Unreleased](https://github.com/ouch-org/ouch/compare/NEW_VERSION...HEAD)

  ### New Features
  ### Improvements
  ### Bug Fixes
  ### Tweaks

  ## [NEW_VERSION](https://github.com/ouch-org/ouch/compare/PREVIOUS_VERSION...NEW_VERSION)
  ```
2. Bump version in Cargo.toml
3. Run `cargo c` to update Cargo.lock
4. Commit all of that
5. `git push`
6. `git tag NEW_VERSION`
7. `git push --tags`
8. head to https://github.com/ouch-org/ouch/actions and wait for the action
9. Go to https://github.com/ouch-org/ouch/releases, edit the release
10. change from pre-release to for-real release
11. Download assets and check their structure
12. nitpick the changelog for some time
13. release to crates.io with `cargo publish`
14. click "release" on GitHub
