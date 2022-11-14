import { parseFrontmatter } from "../../lib/markdown";
import SharedHead from "../SharedHead";

export default async function Head() {
  const frontmatter = await parseFrontmatter("content/work.md");
  return (
    <SharedHead
      title={frontmatter.title}
      description={frontmatter.description}
      path="/work"
      ogType="website"
    />
  );
}
