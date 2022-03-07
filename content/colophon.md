
+++
title = "Colophon"
description = "The Colophon of arne.me"
+++

This website was first published on June 19th 2021 near 
[Frankfurt, Germany](https://frankfurt.de). 
It's developed on a 2017 MacBook Pro with the static site generator
[Zola](https://www.getzola.org).
The code is [hosted on GitHub](https://github.com/bahlo/arne.me).

The primary color is derived from the most recent commit SHA using 
[a Python script](https://github.com/bahlo/arne.me/blob/main/scripts/embed_revision.py).
The current commit is {{ git_sha(include_color=true) }}.

A [Nix](https://nixos.org) flake builds the website on
[GitHub Actions](https://github.com/features/actions).
The resulting files are pushed to an 
[extra GitHub branch](https://github.com/bahlo/arne.me/tree/site), which is
pulled regularly in a cron job on an [Uberspace](https://uberspace.de) account
and served with a domain name provided by [Hover](https://hover.com). 

The [Inter](https://rsms.me/inter/) typeface family is used for text, code is
using [Pragmata Pro](https://fsd.it/shop/fonts/pragmatapro/).

[Plausible Analytics](https://plausible.io) is used, a privacy focused 
alternative to most analytics tools.

Testing was conducted in the latest versions of
[Edge](https://www.microsoft.com/en-us/windows/microsoft-edge/microsoft-edge),
[Chrome](https://www.google.com/chrome/),
[Firefox](https://www.mozilla.org/en-US/firefox/new/), 
and [Safari](http://www.apple.com/safari/).
Any issue you encounter on this website can be submitted as 
[GitHub issues](https://github.com/bahlo/arne.me/issues/new).

A [sitemap](/sitemap.xml) is available and there's an 
[RSS feed](/blog/atom.xml) for blogposts and an [RSS feed](/weekly/atom.xml)
for the Weekly archives.