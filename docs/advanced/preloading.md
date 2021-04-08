# Preloading

## Premise

At the core of preloading is the desire to **reduce** the amount of times Botanist generates `n + 1` queries. Many solutions exist to approach this problem, the most common in GraphQL being [DataLoader](https://github.com/graphql/dataloader). Botanist does not contain a full re-implementation of DataLoader, but does make significant efforts to avoid runaway `n + 1` query creation.

## The n + 1 Problem

Imagine the following hypothetical query:

```graphql
query {
    heros(...) {
        id
        first_name
        enemies {
            id
            first_name
        }
    }
}
```

A fairly typical nested query by all accounts. A naive approach to loading the required data might yield queries looking like:

```sql
SELECT id, first_name FROM heros WHERE id IN (1, 2, 3, 4, 5);
SELECT id, first_name FROM enemies WHERE id = 1;
SELECT id, first_name FROM enemies WHERE id = 2;
SELECT id, first_name FROM enemies WHERE id = 3;
SELECT id, first_name FROM enemies WHERE id = 4;
SELECT id, first_name FROM enemies WHERE id = 5;
```

While not particularly problematic at a small scale, as we increase the number of heros we query for or begin requesting additional nested data from our `enemies`, the number of queries begins to grow exponentially. For every new hero, we end up with a new SQL query.

## Preloading

To avoid the above case, when Botanist loads the query above it will _bulk preload_ descendant models _before_ their individual resolvers run. The loading process looks something like:
1. Load all `Hero`s using a bulk query (like the first line in the above example). This happens any time we use a bulk resolver and doesn't rely on any special preloading logic.
1. Inspect the Juniper `LookAheadSelection` to determine what fields are being queried on these `Hero`s.
    - If these fields are plain fields (i.e they map to columns in the underlying database), do nothing as they should already have been loaded in the first query.
    - If these fields are behind `HasOne` or `HasMany` relationships, take note of the primary key(s) being requested.
1. Load all of the newly discovered primary keys in bulk (per given type). In the above example, this consolidates all `enemies` queries into one query utilizing `IN`.
1. Cache these 'preloaded' models on their parent models (stored in 'hidden' fields of type `Option<T>` or `Option<Vec<T>>`).
1. Rinse and repeat. Attempt this process on all of the models that were just preloaded.

Once preloading is finished, when a field that was preloaded is resolved, we return the model loaded during preloading instead of performing a new query. This solves _most_, but not all `n + 1` query issues. In general, it provides satisfactory performance for my current use.