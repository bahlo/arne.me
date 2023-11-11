---
title: "I18n in Node.js"
description: "Node.js uses ICU (International Components for Unicode) for i18n and only includes English data by default."
published: "2018-11-15"
location: "Frankfurt, Germany"
---

Yesterday we added unit tests for a component that uses the [Intl](https://web.archive.org/web/20200921164547/https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl) API to a frontend project. 
Everything worked flawlessly on our local machines, but it failed on CI. 
The failing tests showed a number formatted in English instead of the expected German format.

<!-- more -->

We use Jest for running the tests, which is running on Node.js. As we found out in the process, Node.js uses ICU (International Components for Unicode) for its i18n support and, if not otherwise specified, only contains English ICU data.

The reason it worked on our local machines was, that they had `node` installed with full ICU data (Homebrew, for example, installs all by default), but the Docker image we used on CI didn't.

## Solution

To have full ICU data available, you can either

- compile the node binary with the flag `--with-intl=full-icu` (see [Options for building Node.js](https://nodejs.org/api/intl.html#intl_options_for_building_node_js)), or
- install the [full-icu](https://www.npmjs.com/package/full-icu) package and use an environment variable (`NODE_ICU_DATA=node_modules/full-icu`) or a command-line flag (`--icu-data-dir=node_modules/full-icu`) to tell node where to find the data (see [Providing ICU data at runtime](https://nodejs.org/api/intl.html#intl_providing_icu_data_at_runtime)).

Since it was only failing on CI, we updated the build script to install the `full-icu` package and export the environment variable. 
Also, in case you wondered, all major browsers support the Intl-API, according to [Can I use](https://caniuse.com/#feat=internationalization).

## Conclusion

If you use any i18n features in your JavaScript application, make sure to include a basic test to be sure all locales you need are supported.
