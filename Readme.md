# Botanist
An experimental [Diesel](http://diesel.rs/) backed GraphQL ORM layer for [Juniper](https://github.com/graphql-rust/juniper).

Botanist will generate a fully featured GraphQL schema from your existing Diesel models and a few bits of additional information you provided. Botanist enables the _fast_ development of rich data models without the need to spend time writing explicit resolvers manually. Of course, writing additional resolvers manually is still supported should you require any more significant application logic.

### Features
- Schema generation from Diesel models
- Bulk load/single load query generation
- Simple `HasOne` / `HasMany` abstractions
- `HasMany` Pagination
- Supports runtime query modification per model (Useful for authorization)
- Create/Update/Delete mutation generation
- Batch model 'preloading' via Juniper `LookAheadSelection`'s

## Basic Usage

This guide assumes you already have a Rust binary setup with Diesel and Juniper.

1. Install Botanist and it's codegen library in `cargo.toml`:
    ```toml
    botanist = "0.1"
    botanist_codegen = "0.1"
    ```

2. Find your Juniper Context and implement the `BotanistContext` trait with something like the following:
    ```rust
    impl BotanistContext for Context {
        type DB = diesel::pg::Pg;
        type Connection = diesel::r2d2::PooledConnection<...>;

        fn get_connection(&self) -> &Self::Connection {
            &self.connection
        }
    }
    ```
    It's important to note that both the `DB` type and the `Connection` type must be defined in the trait implementation. The `DB` type should reference your underlying Diesel database type. The connection type should reference the type of connection you'll provide to Botanist in the `get_connection` function. 


3. Add the `botanist_object` attribute _and_ `table_name` to your Diesel models. It's important to note that the `botanist_attribute` _must_ have a context type specified via `Context = <Your Context Type>`.
    ```rust
    #[botanist_object(Context = Context)]
    #[table_name = "heros"]
    pub struct Hero {
        pub id: Uuid,
        pub name: String,
    }
    ```

4. Finally, add `botanist_query` and `botanist_mutation` to your query and mutation structs respectivly.
    ```rust
    pub struct Query;
    pub struct Mutation;

    #[botanist_query(
        Hero,

        Context = Context,
        PrimaryKey = Uuid,
    )]
    impl Query {}

    #[botanist_mutation(
        Hero,

        Context = Context,
        PrimaryKey = Uuid,
    )]
    impl Mutation {}
    ```
    All types that should be queriable must be listed in `botanist_query`. Types that should have mutations generated for them must be listed in `botanist_mutation`. Both `botanist_query` and `botanist_mutation` must specify the context type (`Context = <Your Context Type>`) and primary key type (`PrimaryKey = <Your Primary Key Type>`). Any resolvers or mutations you explicitly write into the `Query` or `Mutation` struct implementations will be preserved.

## Relationships

Relationships may be expressed with the types `HasOne` and `HasMany`.

### HasOne

```rust
pub struct Hero {
    ...
    pub location: HasOne<Uuid, location, Location>
}
```

`HasOne` fields are an abstraction over Diesel model fields (i.e database forign keys). The `HasOne` type takes the type of the underlying field (in this case a `Uuid`), the referenced models schema and the referenced model itself.

At compile time, this is removed and the Diesel model `Hero` will simple have a `Uuid` location field. However, our GraphQL schema will show `location` to have a type of `Location` and querying for a `Hero`s `location` will return a full location object!

### HasMany

```rust
pub struct Hero {
    ...
    pub enemies: HasMany<enemy, enemy::hero, Enemy>
}
```

`HasMany` fields are pure abstraction - that is they result in no change to the underlying Diesel model at compile time. `HasMany`s only serve to generate resolvers capable of returning more than one model at a time.

In the example above, `HasMany` takes the schema, forign key column and the model itself. The forign key column, `enemy::hero`, is a column in the table we're targeting who's value will match the `id` field of the `Hero` model.

## Query Modifiers
Query modifiers provide you a last minute chance to modify the query of any _root resolver_ before its query is run. These query modifiers provide a convenient time to perform query level authentication. To setup a query modifier, update the `botanist_object` attribute to include `ModifiesQuery = true`. It should look like the following:

```rust
#[botanist_object(Context = Context, ModifiesQuery = true)]
```

Next, implement the trait `QueryModifier`:

```rust
impl<'a> QueryModifier<HeroQuery<'a>, Context> for Hero {
    fn modify_query(query: HeroQuery<'a>, context: &Context) -> HeroQuery<'a> {
        query
    }
}
```
_Note: The `HeroQuery` type is automatically generated for convenience_

Generally,  you'll want to modify the query here instead of just returning it as is. Any standard functions from the Diesel DSL will work here, `HeroQuery` is just a `BoxedSelectQuery`.