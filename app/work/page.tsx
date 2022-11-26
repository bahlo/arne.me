import { parseMarkdown } from "../../lib/markdown";

interface Experience {
  from: number;
  to: number;
  title: string;
  company: string;
  location: string;
  tasks: string[];
}

export default async function Work() {
  const page = await parseMarkdown("content/work.md");
  const { frontmatter, html } = page!;

  return (
    <section>
      <h1>{frontmatter.title}</h1>
      <>
        {frontmatter.experience.map((experience: Experience) => (
          <>
            <h2>
              {experience.title} at {experience.company}
            </h2>
            <span className="details">
              {experience.from} &ndash; {experience.to} &middot;{" "}
              {experience.location}
            </span>
            <ul>
              {experience.tasks.map((task, i) => {
                return <li key={i}>{task}</li>;
              })}
            </ul>
          </>
        ))}
      </>
      <div dangerouslySetInnerHTML={{ __html: html.toString() }} />
    </section>
  );
}
