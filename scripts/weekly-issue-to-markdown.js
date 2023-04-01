import fs from 'fs/promises';
import matter from 'gray-matter';

function getHost(url) {
  const host = new URL(url).host;
  if (host.startsWith("www.")) {
    return host.slice(4);
  }
  return host;
}


(async () => {
  const num = 95; // TODO: Prompt for issue number

  const raw = await fs.readFile(`./src/content/weekly/${num}.md`, 'utf-8');
  const { content, data } = matter(raw);

  console.log(content);

  console.log("## Toot of the Weeek");
  console.log(data.tootOfTheWeek.text.split('\n').map(line => `> ${line}`).join('\n'));
  console.log(`> — [${data.tootOfTheWeek.author}](${data.tootOfTheWeek.url})`);

  data.categories.forEach(category => {
    console.log(`## ${category.title}`);
    category.stories.forEach(story => {
      console.log(`### [${story.title}](https://click.arne.me/?issue=${num}&url=${story.url})`);
      console.log(`${story.readingTimeMinutes} min · ${getHost(story.url)}`);
      console.log(`${story.description}`);
    });
  })
})();
