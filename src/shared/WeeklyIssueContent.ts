// Astro cannot render components, so we'll have to do this, see
// https://docs.astro.build/en/reference/errors/invalid-component-args/
import { marked } from "marked";

function buildAnchor(title) {
  return title.toLowerCase().replace(/[^a-z0-9]+/g, "-");
}

function getHost(url) {
  const u = new URL(url);
  if (u.host == "www.reddit.com") {
    const pathSegements = u.pathname.split("/");
    return u.host.slice(4) + "/" + pathSegements[1] + "/" + pathSegements[2];
  } else if (u.host.startsWith("www.")) {
    return u.host.slice(4);
  }
  return u.host;
}

export default function render(num, frontmatter) {
  const { tootOfTheWeek, tweetOfTheWeek, categories } = frontmatter;

  // In HTML we trust
  return `
    ${
      (tootOfTheWeek &&
        `
    <h2 id="toot-of-the-week">Toot of the Week</h2>
    <blockquote>
      <p>
        ${marked.parse(tootOfTheWeek.text)}
        — <a href="${tootOfTheWeek.url}">${tootOfTheWeek.author}</a>
      </p>
    </blockquote>
    `) ||
      ""
    }
    ${
      (tweetOfTheWeek &&
        `
    <h2 id="tweet-of-the-week">Tweet of the Week</h2>
    <blockquote>
      <p>
        ${marked.parse(tweetOfTheWeek.text)}
        — <a href="${tweetOfTheWeek.url}">${
          tweetOfTheWeek.author
        } on Twitter</a>
      </p>
    </blockquote>
    `) ||
      ""
    }
    ${(categories || [])
      .map(
        (category) => `
    <h2 id=${buildAnchor(category.title)}>${category.title}</h2>
    ${category.stories
      .map(
        (story) => `
    <h3><a href=${
      "https://click.arne.me/?issue=" + num + "&url=" + story.url
    }>${story.title}</a></h3>
    <p class="meta">${
      (story.readingTimeMinutes >= 0
        ? story.readingTimeMinutes + " min &middot; "
        : "") + getHost(story.url)
    }</p>
    ${marked.parse(story.description)}`
      )
      .join("\n")}`
      )
      .join("\n")}
  `;
}
