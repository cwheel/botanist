(window.webpackJsonp=window.webpackJsonp||[]).push([[13],{372:function(t,e,s){"use strict";s.r(e);var a=s(45),n=Object(a.a)({},(function(){var t=this,e=t.$createElement,s=t._self._c||e;return s("ContentSlotsDistributor",{attrs:{"slot-key":t.$parent.slotKey}},[s("h1",{attrs:{id:"text-search"}},[s("a",{staticClass:"header-anchor",attrs:{href:"#text-search"}},[t._v("#")]),t._v(" Text Search")]),t._v(" "),s("p",[t._v("Often, you'll want queries to load models based on field "),s("em",[t._v("similarity")]),t._v(" instead of ID matching. This is particularly common when trying to implement search functionality. Botanist is not a search engine, nor are any of the underlying supported databases. However, Botanist has support for "),s("em",[t._v("basic")]),t._v(" searching which can often be good enough for simple model filtering and matching.")]),t._v(" "),s("h2",{attrs:{id:"enabling-text-search"}},[s("a",{staticClass:"header-anchor",attrs:{href:"#enabling-text-search"}},[t._v("#")]),t._v(" Enabling Text Search")]),t._v(" "),s("p",[t._v("In your "),s("code",[t._v("botanist_query")]),t._v(" declaration, update your type to contain a "),s("code",[t._v("searchable")]),t._v(" key mapping to a tuple of "),s("em",[t._v("text")]),t._v(" ("),s("code",[t._v("VARCHAR")]),t._v(", "),s("code",[t._v("TEXT")]),t._v(" etc.) fields that you'd like to enable searching for. For example, updating the "),s("code",[t._v("Hero")]),t._v(" type to support searching on the "),s("code",[t._v("name")]),t._v(" field and the "),s("code",[t._v("hometown")]),t._v(" field would look like:")]),t._v(" "),s("div",{staticClass:"language-rust extra-class"},[s("pre",{pre:!0,attrs:{class:"language-rust"}},[s("code",[s("span",{pre:!0,attrs:{class:"token attribute attr-name"}},[t._v("#[botanist_query(\n    Hero(\n        searchable = (name, hometown)\n        all = true\n    )\n\n    Context = Context,\n    PrimaryKey = Uuid,\n)]")]),t._v("\n")])])]),s("div",{staticClass:"custom-block tip"},[s("p",{staticClass:"custom-block-title"},[t._v("Note")]),t._v(" "),s("p",[t._v("You must set the "),s("code",[t._v("all")]),t._v(" key to "),s("code",[t._v("true")]),t._v(" here. This will enable your models to be fetched without knowing the exact ID. This is enforced for searching as models are returned based on field matches instead of ID matches.")])]),t._v(" "),s("p",[t._v("If you fire up your application and inspect your schema, you'll see that the multi-select query for "),s("code",[t._v("Hero")]),t._v(" (i.e "),s("code",[t._v("heros(...)")]),t._v(") now has an optional "),s("code",[t._v("query")]),t._v(" argument:")]),t._v(" "),s("div",{staticClass:"language-graphql extra-class"},[s("pre",{pre:!0,attrs:{class:"language-graphql"}},[s("code",[s("span",{pre:!0,attrs:{class:"token attr-name"}},[t._v("heros")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("(")]),s("span",{pre:!0,attrs:{class:"token attr-name"}},[t._v("ids")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" "),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("[")]),t._v("Uuid"),s("span",{pre:!0,attrs:{class:"token operator"}},[t._v("!")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("]")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(",")]),t._v(" "),s("span",{pre:!0,attrs:{class:"token attr-name"}},[t._v("limit")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" Int"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(",")]),t._v(" "),s("span",{pre:!0,attrs:{class:"token attr-name"}},[t._v("offset")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" Int"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(",")]),t._v(" "),s("span",{pre:!0,attrs:{class:"token attr-name"}},[t._v("query")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" HerosQuery"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(")")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" "),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("[")]),t._v("Hero"),s("span",{pre:!0,attrs:{class:"token operator"}},[t._v("!")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("]")]),s("span",{pre:!0,attrs:{class:"token operator"}},[t._v("!")]),t._v("\n")])])]),s("p",[t._v("Inspecting the schema further, you'll see the definition for "),s("code",[t._v("HerosQuery")]),t._v(" looks like:")]),t._v(" "),s("div",{staticClass:"language-graphql extra-class"},[s("pre",{pre:!0,attrs:{class:"language-graphql"}},[s("code",[s("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("input")]),t._v(" HerosQuery "),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("{")]),t._v("\n    "),s("span",{pre:!0,attrs:{class:"token attr-name"}},[t._v("name")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" String\n    "),s("span",{pre:!0,attrs:{class:"token attr-name"}},[t._v("hometown")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" String\n"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("}")]),t._v("\n")])])]),s("p",[t._v("Any field specified in the "),s("code",[t._v("searchable")]),t._v(" tuple will appear in the query input type. Fields in this query are optional and as many or as few as you'd like may be set for any given query.")]),t._v(" "),s("h2",{attrs:{id:"basic-queries"}},[s("a",{staticClass:"header-anchor",attrs:{href:"#basic-queries"}},[t._v("#")]),t._v(" Basic Queries")]),t._v(" "),s("p",[t._v("In general, search queries are implemented using basic, case insensitive like queries. These results are returned in any order the database sees fit. Queries will generally take the form of:")]),t._v(" "),s("div",{staticClass:"language-sql extra-class"},[s("pre",{pre:!0,attrs:{class:"language-sql"}},[s("code",[s("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("WHERE")]),t._v(" field1 ILIKE "),s("span",{pre:!0,attrs:{class:"token string"}},[t._v('"<query>"')]),t._v("\n   "),s("span",{pre:!0,attrs:{class:"token operator"}},[t._v("OR")]),t._v(" field2 ILIKE "),s("span",{pre:!0,attrs:{class:"token string"}},[t._v('"<query>"')]),t._v("\n   "),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(".")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(".")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(".")]),t._v("\n")])])]),s("p",[t._v("If you're using Postgres as your backing database, it's recommended that you read on to the following section for an improved search experience.")]),t._v(" "),s("h2",{attrs:{id:"postgres-prefix-queries"}},[s("a",{staticClass:"header-anchor",attrs:{href:"#postgres-prefix-queries"}},[t._v("#")]),t._v(" Postgres Prefix Queries")]),t._v(" "),s("div",{staticClass:"custom-block warning"},[s("p",{staticClass:"custom-block-title"},[t._v("Warning")]),t._v(" "),s("p",[t._v("These queries are "),s("em",[t._v("only")]),t._v(" generated for Postgres. This section will not work on for any other database.")])]),t._v(" "),s("p",[t._v("As Postgres supports full text search, Botanist can generate some more useful queries when operating on a Postgres database. In particular, Botanist contains a basic prefix match search implementation. To get started with prefix search, enable the "),s("code",[t._v("postgres_prefix_search")]),t._v(" feature for both "),s("code",[t._v("botanist")]),t._v(" and "),s("code",[t._v("botanist_codegen")]),t._v(". Queries will now perform prefix matching and will not rely exclusively on case-insensitive like ("),s("code",[t._v("ILIKE")]),t._v(") anymore.")]),t._v(" "),s("p",[t._v("In general, Botanist will utilize the "),s("code",[t._v("to_tsvector")]),t._v(", "),s("code",[t._v("to_tsquery")]),t._v(" and "),s("code",[t._v("position")]),t._v(" functions with the "),s("code",[t._v("@@")]),t._v(" (match) operator. A query on a single field will look something like:")]),t._v(" "),s("div",{staticClass:"language-sql extra-class"},[s("pre",{pre:!0,attrs:{class:"language-sql"}},[s("code",[s("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("WHERE")]),t._v("\n\tto_tsvector"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("(")]),t._v("field1"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(")")]),t._v(" @@ to_tsquery"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("(")]),s("span",{pre:!0,attrs:{class:"token string"}},[t._v("'<query>:*'")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(")")]),t._v("\n"),s("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("ORDER")]),t._v(" "),s("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("BY")]),t._v("\n\tfield1 ILIKE "),s("span",{pre:!0,attrs:{class:"token string"}},[t._v("'<query>%'")]),t._v(" "),s("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("DESC")]),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(",")]),t._v("\n\tposition"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("(")]),s("span",{pre:!0,attrs:{class:"token string"}},[t._v("'<query>'")]),t._v(" "),s("span",{pre:!0,attrs:{class:"token operator"}},[t._v("in")]),t._v(" field1"),s("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(")")]),t._v(" "),s("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("ASC")]),t._v("\n")])])]),s("ul",[s("li",[t._v("First, the field in question ("),s("code",[t._v("field1")]),t._v(") is converted to a text-search vector.")]),t._v(" "),s("li",[t._v("The vector is then matched against the text-search query (the user provided query with "),s("code",[t._v(":*")]),t._v(" to indicate it should be treated as a prefix)")]),t._v(" "),s("li",[t._v("The results are then ordered:\n"),s("ul",[s("li",[t._v("First, by results with an exact prefix match (the string starts with the prefix)")]),t._v(" "),s("li",[t._v("Next, by the position of the match within the result. Matches where the position is closer to the front rank higher.")])])])])])}),[],!1,null,null,null);e.default=n.exports}}]);