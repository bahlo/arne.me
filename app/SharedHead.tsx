interface Props {
  title?: string,
  description: string,
  path: string,
  ogType?: 'website' | 'article';
}

export default function Head({title, description, path, ogType = 'website'}: Props) {
  if (!title) {
    title = "Arne Bahlo"
  } else {
    title = title + " — Arne Bahlo"
  }

  return (
    <>
      <title>{title}</title>
      <meta name="description" content={description} />
      <meta property="og:title" content={title} />
      <meta property="og:description" content={description} />
      <meta property="og:url" content={"https://arne.me"+path} />
      <meta property="og:type" content={ogType} />
    </>
  )
}