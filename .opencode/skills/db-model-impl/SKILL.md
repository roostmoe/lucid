---
name: db-model-impl
description: Finish implementing the logic for a database model written by a developer.
license: Apache-2.0
compatibility: opencode
metadata:
  audience: maintainers
  workflow: github
---

## What I do

- Create a storage filter struct for the created model
- Create a `Store` trait for the model (i.e. `HostStore`)
- Implement the new `Store` trait for all `Storage` trait implementations
- Add any relevant permissions to the `Permissions` enum in `lucid-common`

## When to use me

Use this when you are tasked with finishing implementation of a database model,
or when writing a _new_ database model.
