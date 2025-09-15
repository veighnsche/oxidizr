# Switchyard Book

This mdBook documents the `switchyard` library, following the documentation plan.

Build locally (requires mdBook):

```bash
# Install once
cargo install mdbook mdbook-linkcheck mdbook-mermaid

# Build
mdbook build

# Serve with live reload
mdbook serve -n 127.0.0.1 -p 3000
```

The book sources live under `src/`. Update content alongside code changes and SPEC updates.
