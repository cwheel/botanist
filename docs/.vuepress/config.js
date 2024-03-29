module.exports = {
    title: '🌿 Botanist',
    description: 'A Rust library for auto-magically generating GraphQL schemas from database models',
    theme: 'default-prefers-color-scheme',
    base: '/botanist/',

    themeConfig: {
        editLinks: true,
        editLinkText: 'Edit this page on GitHub',
        lastUpdated: 'Last Updated',
        repo: 'cwheel/botanist',

        sidebar: [
            ['/', 'Overview'],
            {
                title: 'Getting Started',
                collapsable: false,
                children: [ 'intro/basics', 'intro/schema', 'intro/text_search' ]
            },
            {
                title: 'Relationships',
                collapsable: false,
                children: [ 'relationships/has_one', 'relationships/has_many' ]
            },
            {
                title: 'Advanced',
                collapsable: false,
                children: [ 'advanced/query_modifier', 'advanced/query_options', 'advanced/preloading' ]
            }
        ]
    },
}