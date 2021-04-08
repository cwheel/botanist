(window.webpackJsonp=window.webpackJsonp||[]).push([[11],{369:function(t,s,e){"use strict";e.r(s);var a=e(45),n=Object(a.a)({},(function(){var t=this,s=t.$createElement,e=t._self._c||s;return e("ContentSlotsDistributor",{attrs:{"slot-key":t.$parent.slotKey}},[e("h1",{attrs:{id:"basic-usage"}},[e("a",{staticClass:"header-anchor",attrs:{href:"#basic-usage"}},[t._v("#")]),t._v(" Basic Usage")]),t._v(" "),e("p",[t._v("This guide assumes you already have a Rust binary setup with Diesel and Juniper. For guidiance on setting up either, see their respective docs.")]),t._v(" "),e("ol",[e("li",[e("p",[t._v("Install Botanist and it's codegen library in "),e("code",[t._v("cargo.toml")]),t._v(":")]),t._v(" "),e("div",{staticClass:"language-toml extra-class"},[e("pre",{pre:!0,attrs:{class:"language-toml"}},[e("code",[e("span",{pre:!0,attrs:{class:"token key property"}},[t._v("botanist")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("=")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token string"}},[t._v('"0.1"')]),t._v("\n"),e("span",{pre:!0,attrs:{class:"token key property"}},[t._v("botanist_codegen")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("=")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token string"}},[t._v('"0.1"')]),t._v("\n")])])])]),t._v(" "),e("li",[e("p",[t._v("Find your Juniper Context and implement the "),e("code",[t._v("BotanistContext")]),t._v(" trait with something like the following:")]),t._v(" "),e("div",{staticClass:"language-rust extra-class"},[e("pre",{pre:!0,attrs:{class:"language-rust"}},[e("code",[e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("impl")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token class-name"}},[t._v("BotanistContext")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("for")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token class-name"}},[t._v("Context")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("{")]),t._v("\n    "),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("type")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token constant"}},[t._v("DB")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token operator"}},[t._v("=")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token namespace"}},[t._v("diesel"),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("::")]),t._v("pg"),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("::")])]),e("span",{pre:!0,attrs:{class:"token class-name"}},[t._v("Pg")]),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(";")]),t._v("\n    "),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("type")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token class-name"}},[t._v("Connection")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token operator"}},[t._v("=")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token namespace"}},[t._v("diesel"),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("::")]),t._v("r2d2"),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("::")])]),e("span",{pre:!0,attrs:{class:"token class-name"}},[t._v("PooledConnection")]),e("span",{pre:!0,attrs:{class:"token operator"}},[t._v("<")]),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("...")]),e("span",{pre:!0,attrs:{class:"token operator"}},[t._v(">")]),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(";")]),t._v("\n\n    "),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("fn")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token function-definition function"}},[t._v("get_connection")]),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("(")]),e("span",{pre:!0,attrs:{class:"token operator"}},[t._v("&")]),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("self")]),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(")")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("->")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token operator"}},[t._v("&")]),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("Self")]),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("::")]),e("span",{pre:!0,attrs:{class:"token class-name"}},[t._v("Connection")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("{")]),t._v("\n        "),e("span",{pre:!0,attrs:{class:"token operator"}},[t._v("&")]),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("self")]),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(".")]),t._v("connection\n    "),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("}")]),t._v("\n"),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("}")]),t._v("\n")])])]),e("div",{staticClass:"custom-block tip"},[e("p",{staticClass:"custom-block-title"},[t._v("Types")]),t._v(" "),e("p",[t._v("It's important to note that both the "),e("code",[t._v("DB")]),t._v(" type and the "),e("code",[t._v("Connection")]),t._v(" type must be defined in the trait implementation. The "),e("code",[t._v("DB")]),t._v(" type should reference your underlying Diesel database type (in this example Postgres/"),e("code",[t._v("Pg")]),t._v("). The connection type should reference the type of connection you'll provide to Botanist in the "),e("code",[t._v("get_connection")]),t._v(" function (in this example a type of "),e("code",[t._v("PooledConnection")]),t._v(").")])])]),t._v(" "),e("li",[e("p",[t._v("Add the "),e("code",[t._v("botanist_object")]),t._v(" attribute "),e("em",[t._v("and")]),t._v(" "),e("code",[t._v("table_name")]),t._v(" to your Diesel models.")]),t._v(" "),e("div",{staticClass:"custom-block tip"},[e("p",{staticClass:"custom-block-title"},[t._v("Note")]),t._v(" "),e("p",[t._v("It's important to note that the "),e("code",[t._v("botanist_attribute")]),t._v(" "),e("em",[t._v("must")]),t._v(" have a context type specified via "),e("code",[t._v("Context = <Your Context Type>")]),t._v(".")])]),t._v(" "),e("div",{staticClass:"language-rust extra-class"},[e("pre",{pre:!0,attrs:{class:"language-rust"}},[e("code",[e("span",{pre:!0,attrs:{class:"token attribute attr-name"}},[t._v("#[botanist_object(Context = Context)]")]),t._v("\n"),e("span",{pre:!0,attrs:{class:"token attribute attr-name"}},[t._v("#[table_name = "),e("span",{pre:!0,attrs:{class:"token string"}},[t._v('"heros"')]),t._v("]")]),t._v("\n"),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("pub")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("struct")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token type-definition class-name"}},[t._v("Hero")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("{")]),t._v("\n    "),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("pub")]),t._v(" id"),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token class-name"}},[t._v("Uuid")]),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(",")]),t._v("\n    "),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("pub")]),t._v(" name"),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(":")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token class-name"}},[t._v("String")]),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(",")]),t._v("\n"),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("}")]),t._v("\n")])])])]),t._v(" "),e("li",[e("p",[t._v("Finally, add "),e("code",[t._v("botanist_query")]),t._v(" and "),e("code",[t._v("botanist_mutation")]),t._v(" to your query and mutation structs respectively.")]),t._v(" "),e("div",{staticClass:"language-rust extra-class"},[e("pre",{pre:!0,attrs:{class:"language-rust"}},[e("code",[e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("pub")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("struct")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token type-definition class-name"}},[t._v("Query")]),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(";")]),t._v("\n"),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("pub")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("struct")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token type-definition class-name"}},[t._v("Mutation")]),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v(";")]),t._v("\n\n"),e("span",{pre:!0,attrs:{class:"token attribute attr-name"}},[t._v("#[botanist_query(\n    Hero,\n\n    Context = Context,\n    PrimaryKey = Uuid,\n)]")]),t._v("\n"),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("impl")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token class-name"}},[t._v("Query")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("{")]),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("}")]),t._v("\n\n"),e("span",{pre:!0,attrs:{class:"token attribute attr-name"}},[t._v("#[botanist_mutation(\n    Hero,\n\n    Context = Context,\n    PrimaryKey = Uuid,\n)]")]),t._v("\n"),e("span",{pre:!0,attrs:{class:"token keyword"}},[t._v("impl")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token class-name"}},[t._v("Mutation")]),t._v(" "),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("{")]),e("span",{pre:!0,attrs:{class:"token punctuation"}},[t._v("}")]),t._v("\n")])])]),e("p",[t._v("All types (Diesel models) that should be queryable must be listed in "),e("code",[t._v("botanist_query")]),t._v(". Types (Diesel models) that should have mutations generated for them must be listed in "),e("code",[t._v("botanist_mutation")]),t._v(". Both "),e("code",[t._v("botanist_query")]),t._v(" and "),e("code",[t._v("botanist_mutation")]),t._v(" must specify the context type ("),e("code",[t._v("Context = <Your Context Type>")]),t._v(") and primary key type ("),e("code",[t._v("PrimaryKey = <Your Primary Key Type>")]),t._v("). Any resolvers or mutations you explicitly write into the "),e("code",[t._v("Query")]),t._v(" or "),e("code",[t._v("Mutation")]),t._v(" struct implementations will be preserved.")])])])])}),[],!1,null,null,null);s.default=n.exports}}]);