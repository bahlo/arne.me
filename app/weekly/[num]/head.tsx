import { parseFrontmatter } from "../../../lib/markdown";
import SharedHead from "../../SharedHead";

export default async function Head({
  params: { num },
}: {
  params: { num: string };
}) {
  const frontmatter = await parseFrontmatter("content/weekly/" + num + ".md");
  return (
    <SharedHead
      title={frontmatter.title}
      description={`Issue #${num} of Arne's Weekly`}
      path={"/weekly/" + num}
      ogType="article"
    />
  );
}
