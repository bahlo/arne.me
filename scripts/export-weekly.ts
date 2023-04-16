import fs from "fs/promises";
import matter from "gray-matter";
import readline from "readline";

function getHost(url: string): string {
  const host = new URL(url).host;
  if (host.startsWith("www.")) {
    return host.slice(4);
  }
  return host;
}

interface Markdown {
  content: string;
  data: Frontmatter;
}

interface Frontmatter {
  tootOfTheWeek: TootOfTheWeek;
  categories: Category[];
}

interface TootOfTheWeek {
  text: string;
  author: string;
  url: string;
}

interface Category {
  title: string;
  stories: Story[];
}

interface Story {
  title: string;
  url: string;
  readingTimeMinutes: number;
  description: string;
}

(async () => {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stderr,
  });
  const num = await new Promise((resolve) =>
    rl.question("Issue number: ", resolve)
  );
  rl.close();

  const raw = await fs.readFile(`./src/content/weekly/${num}.md`, "utf-8");
  const { content, data }: Markdown = matter(raw) as any;

  console.log(content);

  console.log("## Toot of the Week");
  console.log(
    data.tootOfTheWeek.text
      .split("\n")
      .map((line) => `> ${line}`)
      .join("\n")
  );
  console.log(
    `> — [${data.tootOfTheWeek.author}](${data.tootOfTheWeek.url})\n`
  );

  data.categories.forEach((category) => {
    console.log(`## ${category.title}`);
    category.stories.forEach((story) => {
      console.log(
        `### [${story.title}](https://click.arne.me/?issue=${num}&url=${story.url})`
      );
      console.log(`${story.readingTimeMinutes} min · ${getHost(story.url)}`);
      console.log();
      console.log(`${story.description}`);
    });
  });
})();
