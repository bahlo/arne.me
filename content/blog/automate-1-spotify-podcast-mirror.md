---
title: "Automate #1: Spotify Podcast Mirror"
published: "2019-01-05"
updated: "2023-11-11"
description: "How I created an Automator application, which records the latest episode of a Spotify podcast, fills out metadata and generates a file for metadata for a podcast client to subscribe to."
location: "Frankfurt, Germany"
---

This is the first post of my series _Automate_.

In this post I describe how I created an Automator application, which will record the latest episode of a Spotify podcast, fill out metadata like title and description and generate a file for metadata for a podcast client to subscribe to.

<!-- more -->

## Preparation

If you want to follow along, you need a machine with macOS and the following installed:

- Spotify
- [Audio Hijack](https://www.rogueamoeba.com/audiohijack/) ($77, sorry)
- [jq](https://stedolan.github.io/jq)
- [Golang](https://golang.org/)

You also need a bit of knowledge in Apple Script, Bash scripting and JSON and be comfortable in a terminal.

### Create Audio Hijack Session

Audio Hijack is an application that can do complex audio pipelines, often used for recording podcasts or live streaming. We use this to capture the output of Spotify to a file. Audio Hijack has a concept of _sessions_, meaning various saved pipelines.

To prepare things, start Audio Hijack and create a new session based on the `Application Audio` template.
This post assumes you only have on session. If you have multiple, make sure it's at the first position. In the session, select Spotify as Application and change the Recorder to 128 kbps (or whatever you prefer). I recommend disabling the Output Device, but it's helpful if you want to debug something. After saving, it should look similar to this:

<picture>
  <source srcset="/blog/automate-1-spotify-podcast-mirror/audio_hijack_session.avif" type="image/avif" />
  <img src="/blog/automate-1-spotify-podcast-mirror/audio_hijack_session.png" alt="An AudioHijack session with Spotify and a Recorder" />
</picture>

### Spotify Credentials

We need various metadata about the latest show, like title, Spotify uri or duration.
To get those, we use the [Spotify Web API](https://developer.spotify.com/documentation/web-api/). You need to create a Spotify App by going to the [developer dashboard](https://developer.spotify.com/dashboard) and create a client id.
Fill out the fields and you'll get a `Client ID` and a `Client Secret`.

### Spotify ID

We also need the Spotify ID of the podcast you want to mirror. To get that, click the three dots on the podcast page in Spotify, select `Share` and `Copy Spotify URI`.
The uri looks something like `spotify:show:abcdef`, where `abcdef` is your Spotify ID.

### Folder Setup

Lastly, there needs to be a folder where the podcast lives. We use a tool in our last step to generate the RSS feed, so we can subscribe to the podcast from our favourite podcast app. Fot the tool to work, we need to have a `podcast.toml` similar to this [example](https://github.com/bahlo/toml-podcast/blob/dffe266169567348cb5049f0bdc8c09a56e10879/examples/podcast.toml), but without the `[[episodes]]` parts in our folder.
There also needs to be an empty subfolder called `dist/`.

## Let's do this

### 1. Create an Automator Project

Open the Automator app and choose `New Document`.
Then click on `Application` to build a `.app` file you can launch later.

### 2. Get Latest Episode Metadata

We need the following attributes of the latest episode:

- Spotify URI
- Duration
- Name
- Description
- Release date

We use Bash to get the data, so drag a new Run Shell Script onto your workflow with the following contents:

```sh
#!/bin/bash

# Exit the script if any command returns an error code.
set -e

# Define variables
PODCAST_PATH='/path/to/podcast/folder'
SPOTIFY_ID='abcdef'
AUTH_HEADER='abcdef123456' # base64 encoded `<client id>:<client secret>`

# Get Spotify access token
TOKEN_RES=$(curl -s -X POST 'https://accounts.spotify.com/api/token' \
	-H Authorization:"Basic $AUTH_HEADER" --data 'grant_type=client_credentials')
ACCESS_TOKEN=$(echo $TOKEN_RES | /usr/local/bin/jq -r .access_token)

# Get show
SHOW_RES=$(curl -s "https://api.spotify.com/v1/shows/$SPOTIFY_ID" \
	-H Authorization:"Bearer $ACCESS_TOKEN")
# Get first (latest) item
ITEM=$(echo "$SHOW_RES" | /usr/local/bin/jq '.episodes.items[0]')
# Get release date
RELEASE_DATE=$(echo "$ITEM" | /usr/local/bin/jq -r .release_date)

# If the episode already exists, exit early
if [ -f "$PODCAST_PATH/dist/episodes/$RELEASE_DATE.mp3" ]; then
	(>&2 echo "Episode already exists")
	exit 1
fi

# Get and echo metadata, each line will be a parameter to the next action
# in the Automator workflow.
echo "$ITEM" | /usr/local/bin/jq -r .uri
echo "$ITEM" | /usr/local/bin/jq -r .duration_ms
echo "$ITEM" | /usr/local/bin/jq -r .name
echo "$ITEM" | /usr/local/bin/jq -r .description
echo "$RELEASE_DATE"
```

We need the metadata again later, so save it in a variable by dragging the `Set value of variable` below the Bash action and giving it a name (e.g. `Metadata`).

### 3. Start the Recording Process

Now we get to the messy part. Drag a `Run AppleScript` action to the end of the workflow and paste the following code.

Read the comments (lines starting with `--`) to know what it's doing. Make sure the checkbox next to Automator in System Settings → Security & Privacy → Accessibility) is checked, otherwise you will get an error and the keyboard shortcuts won't work.

```applescript
on run {input, parameters}
	set uri to item 1 of input
	set duration_ms to item 2 of input
	set title to item 3 of input
	set description to item 4 of input
	set release_date to item 5 of input

	-- Prepare Spotify
	tell application "Spotify"
		-- Start spotify
		activate

		-- Start playing our episode
		play track uri

		-- Make sure we start at 0s
		-- If we are at 0s, this will skip to the previous track
		delay 1
		set episode_id to id of current track
		previous track
		pause
		delay 1
		if episode_id is not id of current track then
			play track uri
			pause
		end if
	end tell

	-- Start recording
	tell application "Audio Hijack"
		-- Start Audio Hijack
		activate

		tell application "System Events"
			-- CMD + 1 to open Sessions
			keystroke "1" using command down
			-- Arrow down to select the first one
			key code 124
			-- Open session
			keystroke "o" using command down
			-- Start recording
			keystroke "r" using command down
		end tell
	end tell

	-- Play track
	tell application "Spotify"
		-- Bring to foreground
		activate

		-- Start playing our track again
		play track uri

		-- Wait the length of our episode
		delay duration_ms / 1000

		-- Press pause
		pause

		-- Quit spotify
		quit
	end tell

	tell application "Audio Hijack"
		-- Bring Audio Hijack to foreground
		activate

		tell application "System Events"
			-- Stop recording
			delay 1
			keystroke "r" using command down
		end tell

		-- Quit Audio Hijack
		quit
	end tell
end run
```

### 4. Save Release Date to it's Own Variable

We use the publish date of the episode as filename, so we need to get this first. Drag the action _Get Value of Variable_ into your workflow and select the `Metadata` variable (or whatever you called it). After that, drag a _Run AppleScript_ action at the end with the following contents.

```applescript
on run {input, parameters}
	-- Return the fifth parameter of our Metadata variable,
	-- which is the release date
	return item 5 of input
end run
```

Set the variable `ReleaseDate` using a _Set Value of Variable_ action.

### 5. Move File to Destination

Use a _Find Finder_ items action to search the destination folder of your Audio Hijack session for files with all of the following attributes:

- Kind is music
- Date created is today

Then drag a _Run AppleScript_ with the following contents into your workflow to get the first item (which will be the latest).

```applescript
on run {input, parameters}
	# Get latest item
	return item 1 of input
end run
```

After that, rename the file using _Rename Finder Items_. Choose Name `Single Item` and set `Basename only` to `ReleaseDate`.

Then move the renamed file to `podcast-folder/dist/episodes/` with a _Move Finder Items_ action.

### 6. Metadata and Deployment

Now that we have our recording, we have to generate the RSS feed with the metadata so podcast clients can display the episodes in a nice list. After that we upload both the feed and the episode.

Use _Get Value of Variable_ to get the `Metadata` variable again.

To generate the `feed.xml`, we append the metadata of an episode to a simple TOML file. I wrote a small tool for this, which you can find here: [toml-podcast](https://web.archive.org/web/20200923091609/https://github.com/bahlo/toml-podcast).
After generating the `feed.xml`, we use the [AWS CLI](https://aws.amazon.com/cli) deploy it and the newly recorded episode to an AWS S3 bucket.

Add a _Run Shell Script_ action with the following contents:

```sh
#!/bin/bash

set -e

PODCAST_PATH='/path/to/podcast/folder'
URI="$2"
DURATION_MS="$3"
TITLE="$4"
# For description, we have to escape double quotes ("), because the TOML strings
# use double quotes as well.
# If the TOML is invalid, toml-podcast will crash.
DESCRIPTION=$(echo "$5" | sed "s/\"/\\\\\"/g")
RELEASE_DATE="$6"

# Add episode metadat to the end to podcast.toml
cat << EOT >> "$PODCAST_PATH/podcast.toml"
[[episodes]]
title="$TITLE"
file="$RELEASE_DATE.mp3"
date="$RELEASE_DATE"
description="$DESCRIPTION"

EOT

# Build feed.xml using toml-pdocast
cd $PODCAST_PATH
/Users/arnebahlo/Developer/go/bin/toml-podcast
# Your $GOBIN will look different, find out your path by typing `which toml-podcast`
# in your terminal

# Set AWS credentials (replace with your real credentials)
export AWS_ACCESS_KEY_ID="ABCDEF"
export AWS_SECRET_ACCESS_KEY="GHIJKL"

# Upload episodes + feed.xml
cd dist/
/usr/local/bin/aws s3 sync episodes s3://fuf-mirror/episodes
/usr/local/bin/aws s3 cp --cache-control max-age=600 feed.xml s3://fuf-mirror/feed.xml
```

I use a `Cache-Control` header because I serve the podcast via a CloudFront, the AWS CDN and I want the `feed.xml` to be at most 10 minutes old.

## Conclusion

Now recording the latest episode of your favourite Spotify podcast takes one click. If you have a spare Mac you could even start it automatically matching the release cycle of the podcast.
