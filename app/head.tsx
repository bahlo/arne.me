export default async function Head({ params }: any) {
  return (
    <>
      <title>Arne Bahlo</title>
      <meta charSet="utf-8" />
      <meta http-equiv="X-UA-Compatible" content="IE=edge" />
      <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no" />
      <meta name="apple-mobile-web-app-capable" content="yes" />
      <meta name="theme-color" content="#82f2f3" />
      <meta name="description" content="The personal website of Arne Bahlo." />
      <meta name="author" content="Arne Bahlo" />
      <meta property="og:site_name" content="Arne Bahlo" />
      <meta property="og:title" content="Arne Bahlo" />
      <meta property="og:type" content="website" />
      <meta property="og:url" content="https://arne.me/" />
      <meta property="og:locale" content="en_US" />
      <meta property="og:description" content="The personal website of Arne Bahlo." />
      <link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png" />
      <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png" />
      <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png" />
      <link rel="manifest" href="/site.webmanifest" />
      <link rel="mask-icon" href="/mask-icon.svg" color="#111111" />
      <link rel="alternate" type="application/rss+xml" title="Blog posts" href="https://arne.me/blog/atom.xml" />
      <link rel="alternate" type="application/rss+xml" title="Arne’s  Weekly archive" href="https://arne.me/weekly/atom.xml" />
      <link href="mailto:hey@arne.me" rel="me authn" />
    </>
  )
}