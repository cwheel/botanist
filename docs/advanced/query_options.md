# Query Options

For any Type (Diesel model) that's queryable, there are additional options that may be specified. To specify these options, append parenthese to the end of the Type and specify options in the form of `option = value`.

```rust
#[botanist_query(
    Hero(
        some_option = true
    ),

    Context = Context,
    PrimaryKey = Uuid,
)]
```

## all

The `all` option changes the type of `ids` in a pluralized resolver from `[Type!]!` to `[Type!]!`. This makes specifying an explicit array of ids optional. Use this if you wish to allow for arbitrary pagination of data for a given type.

**Example:**
```rust
Hero(
    all = true
)
```

## plural

The `plural` option accepts a string that overrides Botanists default pluralization of a type. For any type name that follows more complex pluralization rules than appending an `s` to the end of the singular type, specifying this is encouraged.

**Example:**
```rust
Sandwich(
    plural = "Sandwiches"
)
```