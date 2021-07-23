# Text Search

Often, you'll want queries to load models based on field *similarity* instead of ID matching. This is particularly common when trying to implement search functionality. Botanist is not a search engine, nor are any of the underlying supported databases. However, Botanist has support for *basic* searching which can often be good enough for simple model filtering and matching.

## Enabling Text Search

In your `botanist_query` declaration, update your type to contain a `searchable` key mapping to a tuple of *text* (`VARCHAR`, `TEXT` etc.) fields that you'd like to enable searching for. For example, updating the `Hero` type to support searching on the `name` field and the `hometown` field would look like:

```rust
#[botanist_query(
    Hero(
        searchable = (name, hometown)
        all = true
    )

    Context = Context,
    PrimaryKey = Uuid,
)]
```

::: tip Note
You must set the `all` key to `true` here. This will enable your models to be fetched without knowing the exact ID. This is enforced for searching as models are returned based on field matches instead of ID matches.
:::

If you fire up your application and inspect your schema, you'll see that the multi-select query for `Hero` (i.e `heros(...)`) now has an optional `query` argument:

```graphql
heros(ids: [Uuid!], limit: Int, offset: Int, query: HerosQuery): [Hero!]!
```

Inspecting the schema further, you'll see the definition for `HerosQuery` looks like:

```graphql
input HerosQuery {
    name: String
    hometown: String
}
```

Any field specified in the `searchable` tuple will appear in the query input type. Fields in this query are optional and as many or as few as you'd like may be set for any given query.

## Basic Queries

In general, search queries are implemented using basic, case insensitive like queries. These results are returned in any order the database sees fit. Queries will generally take the form of:

```sql
WHERE field1 ILIKE "<query>"
   OR field2 ILIKE "<query>"
   ...
```

If you're using Postgres as your backing database, it's recommended that you read on to the following section for an improved search experience.


## Postgres Prefix Queries

::: warning Warning
These queries are *only* generated for Postgres. This section will not work on for any other database.
:::

As Postgres supports full text search, Botanist can generate some more useful queries when operating on a Postgres database. In particular, Botanist contains a basic prefix match search implementation. To get started with prefix search, enable the `postgres_prefix_search` feature for both `botanist` and `botanist_codegen`. Queries will now perform prefix matching and will not rely exclusively on case-insensitive like (`ILIKE`) anymore.

In general, Botanist will utilize the `to_tsvector`, `to_tsquery` and `position` functions with the `@@` (match) operator. A query on a single field will look something like:

```sql
WHERE
	to_tsvector(field1) @@ to_tsquery('<query>:*')
ORDER BY
	field1 ILIKE '<query>%' DESC,
	position('<query>' in field1) ASC
```

- First, the field in question (`field1`) is converted to a text-search vector.
- The vector is then matched against the text-search query (the user provided query with `:*` to indicate it should be treated as a prefix)
- The results are then ordered:
    - First, by results with an exact prefix match (the string starts with the prefix)
    - Next, by the position of the match within the result. Matches where the position is closer to the front rank higher.