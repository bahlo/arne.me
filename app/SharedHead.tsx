import { buildAbsolutePath } from '../lib/markdown';
import sizeOf from 'image-size';

interface Props {
  title?: string,
  description: string,
  path: string,
  date?: string,
  ogType?: 'website' | 'article';
  ogImage?: {
    src: string,
    alt: string,
  }
}

export default function Head(props: Props) {
  let title = props.title;
  if (!title) {
    title = "Arne Bahlo"
  } else {
    title = title + " — Arne Bahlo"
  }

  let metaArticlePublishedTime = null;
  if (props.date) {
    // TODO: Fix this once we have time formatting
    // articlePublishedTime = <meta property="article:published_time" content="{{ page.date | date(format="%Y-%m-%dT00:00:00+00:00:00")}}">
  }

  let ogImage = null;
  if (props.ogImage) {
    const { width, height } = sizeOf(buildAbsolutePath('public', props.ogImage.src));
    ogImage = <>
      <meta property="og:image" content={`https://arne.me${props.ogImage.src}`} />
      <meta property="og:image:alt" content={props.ogImage.alt} />
      <meta property="og:image:width" content={width?.toString()} />
      <meta property="og:image:height" content={height?.toString()} />
      <meta name="twitter:image" content={`https://arne.me${props.ogImage.src}`} />
      <meta name="twitter:image:alt" content={props.ogImage.alt} />
      <meta name="twitter:card" content="summary_large_image" />
    </>
  }

  return (
    <>
      <title>{title}</title>
      <meta name="description" content={props.description} />
      <meta property="og:title" content={title} />
      <meta property="og:description" content={props.description} />
      <meta property="og:url" content={"https://arne.me"+props.path} />
      <meta property="og:type" content={props.ogType} />
      {metaArticlePublishedTime}
      {ogImage}
    </>
  )
}