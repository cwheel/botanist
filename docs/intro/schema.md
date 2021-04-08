# Generated Schema

Once you've got a basic `Query` and `Mutation` setup, a schema will be generated for you. To understand what resolvers will be generated for any given type, read on.

::: tip Note
The examples on this page use `Uuid` as the primary key type. Your schema will reflect the type you specify for your primary keys.
:::

## Query

Given the example type (Diesel model) `Hero`, the following query resolvers will be generated:

A singular resolver:
```graphql
hero(id: Uuid!): Hero!
```

A bulk resolver (easily interoperable with [Apollo pagination](https://www.apollographql.com/docs/tutorial/queries/#add-pagination-support)):
```graphql
heros(ids: [Uuid!], limit: Int, offset: Int): [Hero!]!
```

::: tip Note
By default, bulk resolvers are required to specify a set of ids (primary keys) they wish to load. If you'd like to disable this behavior for a particular type, see the [all option](/advanced/query_options.html#all).
:::

## Mutation

Given the example type (Diesel model) `Hero`, the following mutation resolvers will be generated:

Create:
```graphql
createHero(input: NewHero!): Hero!
```

Update:
```graphql
updateHero(input: HeroUpdate!): Hero!
```

Delete:
```graphql
deleteHero(id: Uuid!): Hero!
```

Additionally, the following types will be generated:

```graphql
input NewHero {
    ...
}
```
The `New` type will have fields who's optionality match that specified in the Diesel model.

```graphql
input HeroUpdate {
    ...
}
```
All fields of the `Update` type are optional (excluding the primary key).