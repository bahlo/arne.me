import { parseFrontmatter } from '../../../lib/markdown';
import SharedHead from '../../SharedHead';

export default async function Head({ params: { slug }}: { params: { slug: string }}) {
  const frontmatter = await parseFrontmatter('content/blog/'+slug+'.md');
  return <SharedHead title={frontmatter.title} description={frontmatter.description} path={'/blog/'+slug} ogType="article" ogImage={frontmatter.coverImage} />;
}