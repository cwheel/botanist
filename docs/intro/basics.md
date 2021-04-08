# Basic Usage

This guide assumes you already have a Rust binary setup with Diesel and Juniper. For guidiance on setting up either, see their respective docs.

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
    ::: tip Types
    It's important to note that both the `DB` type and the `Connection` type must be defined in the trait implementation. The `DB` type should reference your underlying Diesel database type (in this example Postgres/`Pg`). The connection type should reference the type of connection you'll provide to Botanist in the `get_connection` function (in this example a type of `PooledConnection`).
    :::


3. Add the `botanist_object` attribute _and_ `table_name` to your Diesel models.

    ::: tip Note
    It's important to note that the `botanist_attribute` _must_ have a context type specified via `Context = <Your Context Type>`.
    :::

    ```rust
    #[botanist_object(Context = Context)]
    #[table_name = "heros"]
    pub struct Hero {
        pub id: Uuid,
        pub name: String,
    }
    ```

4. Finally, add `botanist_query` and `botanist_mutation` to your query and mutation structs respectively.
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
    All types (Diesel models) that should be queryable must be listed in `botanist_query`. Types (Diesel models) that should have mutations generated for them must be listed in `botanist_mutation`. Both `botanist_query` and `botanist_mutation` must specify the context type (`Context = <Your Context Type>`) and primary key type (`PrimaryKey = <Your Primary Key Type>`). Any resolvers or mutations you explicitly write into the `Query` or `Mutation` struct implementations will be preserved.