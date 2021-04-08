# Botanist
An experimental [Diesel](http://diesel.rs/) backed GraphQL ORM layer for [Juniper](https://github.com/graphql-rust/juniper). In other words, Botanist can generate a fully featured GraphQL schema from your existing database models without much manual work.

Botanist will generate a fully featured GraphQL schema from your existing Diesel models and a few bits of additional information you provided. Botanist enables the _fast_ development of rich data models without the need to spend time writing explicit resolvers manually. Of course, writing additional resolvers manually is still supported should you require any more significant application logic.

### Features
- Schema generation from Diesel models
- Bulk load/single load query generation
- Simple `HasOne` / `HasMany` abstractions
- `HasMany` Pagination
- Supports runtime query modification per model (Useful for authorization)
- Create/Update/Delete mutation generation
- Batch model 'preloading' via Juniper `LookAheadSelection`'s

### Docs
Read the docs book [here](https://cwheel.github.io/botanist).