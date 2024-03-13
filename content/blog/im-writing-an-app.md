---
title: "I'm Writing an App"
description: "A human, interested in software development, automation and sustainability"
published: "2020-06-12"
updated: "2023-11-11"
location: "Frankfurt, Germany"
---

The last two weeks I've spent quite some time on evenings and weekends to work on an iOS app. 
I won't tell you what it is though, it's way too early for that.

This is the first post in a series and this one is about technologies and tooling.

<!-- more -->

## Technologies

I'm writing a native app in Swift and UIKit.

The decision against using React Native or any other third-party mobile technology was mostly because native feels better (to me), is often faster and gets new features first. Using React Native would have sped up the development process a lot (mostly because I know that technology and my Swift/UIKit knowledge is a little rusty), but I wanted it to be native and I'm in no rush.

The project was initially set up to use SwiftUI as it seems to be the future of iOS development, but I quickly hit some limits. More experienced iOS-Developers would have probably found a way around it, but I feel more at home with UIKit and it's a stable, battle-tested technology.

## Tooling

Spending most of my working time writing Go I've come to value automatic formatting and good linting. For that I'm using [SwiftFormat](https://github.com/nicklockwood/SwiftFormat) and [SwiftLint](https://github.com/realm/SwiftLint), both with the default configuration. You can configure Xcode to automatically run those when building, which I highly recommend.

For dependency management I've started using [Carthage](https://github.com/Carthage/Carthage), mostly because that was the defacto-standard when I last wrote an app. But I've switched over to [Swift Package Manager](https://swift.org/package-manager) because it has great Xcode-integration and is generally stable.

Most iOS developers have experienced their share of .xcodeproj-related merge conflicts or weird diffs. Generally I prefer all files in my project to be human-readable. For that I've used [XcodeGen](https://github.com/yonaskolb/XcodeGen), which allows you to define your project in a simple `project.yml` and ignore your project with these lines in you `.gitignore`:

```
*.xcodeproj
!*.xcodeproj/project.xcworkspace/xcshareddata/swiftpm/Package.resolved
```

That means that on CI, new clones or changes in project.yml you need to install XcodeGen and run `xcodegen generate` to set it up, but for me that's worth it.

Talking about CI, I've set up [Fastlane](https://fastlane.tools/) for linting and testing. It allows easy definition of rules and automatic TestFlight or even AppStore submissions (including taking screenshots), which is awesome. It also has a [ton of plugins](https://docs.fastlane.tools/plugins/available-plugins).

Since Travis CI costs a [small fortune](https://travis-ci.com/plans) for private repositories I've migrated away and now use [GitHub Actions](https://github.com/features/actions).
It took some time to configure it right, so here's my `.github/workflows/ci.yml`, hoping it's useful for some people:

```yaml
name: CI

on: [push]

jobs:
  lint-and-test:
    runs-on: macos-latest
    steps:
      - run: sudo xcode-select -s /Applications/Xcode_11.5.app/Contents/Developer
      - name: Checkout code
        uses: actions/checkout@v2
      - name: Cache Bundler
        uses: actions/cache@v2
        with:
          path: vendor/bundle
          key: $-gems-$
          restore-keys: |
            $-gems-
      - name: Cache Swift Package Manager
        uses: actions/cache@v2
        with:
          path: .build
          key: $-spm-$
          restore-keys: |
            $-spm-
      - name: Install dependencies
        run: |
          bundle update --bundler
          bundle install
          brew install xcodegen swiftlint swiftformat
      - name: Generate Xcode project
        run: xcodegen generate
      - name: Lint
        run: bundle exec fastlane lint
      - name: Test
        run: bundle exec fastlane test
```

That's my setup so far. If you have any questions or suggestions, 
[hit me up](/contact).
