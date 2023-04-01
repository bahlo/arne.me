// Astro cannot render components, so we'll have to do this, see
// https://docs.astro.build/en/reference/errors/invalid-component-args/
import { marked } from 'marked';

function buildAnchor(title) {
  return title.toLowerCase().replace(/[^a-z0-9]+/g, '-');
}

function getHost(url) {
  const host = new URL(url).host;
  if (host.startsWith("www.")) {
    return host.slice(4);
  }
  return host;
}

export default function render(num, frontmatter) {
  const { tootOfTheWeek, categories } = frontmatter

  // In HTML we trust
  return `
    <slot />
    ${tootOfTheWeek && `
    <h2 id="toot-of-the-week">Toot of the Week</h2>
    <blockquote>
      <p>
        ${marked.parse(tootOfTheWeek.text)}
        — <a href="${tootOfTheWeek.url}">${tootOfTheWeek.author}</a>
      </p>
    </blockquote>
    ` || ''}
    ${(categories || []).map(category => `
    <h2 id=${buildAnchor(category.title)}>${category.title}</h2>
    ${category.stories.map(story => `
    <h3 class="no-margin-bottom"><a href=${"https://click.arne.me/?issue="+num+"&url=" + story.url}>${story.title}</a></h3>
    <p class="meta"><em>${(story.readingTimeMinutes >= 0 ? story.readingTimeMinutes + " min &middot; " : "") + getHost(story.url)}</em></p>
    ${marked.parse(story.description)}`).join('\n')}`).join('\n')}
  `
}
