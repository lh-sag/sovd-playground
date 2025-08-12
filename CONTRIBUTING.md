# How to contribute

Welcome to this project. The developers can be reached via [slack](https://app.slack.com/client/T02MS1M89UH/C02MS1M9BH7)

## Conventions

The following conventions are followed within this project.

* [GitHub Flow](https://docs.github.com/en/get-started/using-github/github-flow)
  * Create a feature branch
  * Make changes
  * Create a pull request
  * Handle code reviews
  * Merge pull request
  * An alternative name is `feature branch workflow`. Note: This is _not_ GitFLow.
* [Semantic Versioning](https://semver.org)
  * Version follows the schema MAJOR.MINOR.PATCH for relase tags
  * Optionally a trailing pre-release/build identifier can be added e.g. 1.2.3-rc.1+01
* [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)
  * Template commit message:

    ```text
    <type>[optional scope]: <description>

    [optional body]

    [optional footer(s)]
    ```

  * An optional footer could be a GitHub issue e.g. `Issue: GH-42` or `Issue: eclipse-opensovd/opensovd-core#42`
  * See git commit message [template](.gitmessage)
* Install [pre-commit](https://pre-commit.com/) prior to PR

## Legal considerations

* Sign the Eclipse Contributor Agreement ([ECA](https://www.eclipse.org/legal/eca/)) electronically prior to contributing.
* Newly created files contain a proper license header

  ```rust
  //
  // Copyright (c) 2025 The Contributors to Eclipse OpenSOVD.
  //
  // See the NOTICE file(s) distributed with this work for additional
  // information regarding copyright ownership.
  //
  // This program and the accompanying materials are made available under the
  // terms of the Apache License Version 2.0 which is available at
  // https://www.apache.org/licenses/LICENSE-2.0
  //
  // SPDX-License-Identifier: Apache-2.0
  //

  ```
