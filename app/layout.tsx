/* eslint-disable @next/next/no-head-element */

import "../styles/global.css";
import Footer from "./Footer";
import Header from "./Header";

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html>
      <head>
        <meta charSet="utf-8" />
        <meta httpEquiv="X-UA-Compatible" content="IE=edge" />
        <meta
          name="viewport"
          content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no"
        />
        <meta name="apple-mobile-web-app-capable" content="yes" />
        <meta name="theme-color" content="#82adf3" />
        <meta name="author" content="Arne Bahlo" />
        <meta property="og:site_name" content="Arne Bahlo" />
        <meta property="og:locale" content="en_US" />
        <link
          rel="apple-touch-icon"
          sizes="180x180"
          href="/apple-touch-icon.png"
        />
        <link
          rel="icon"
          type="image/png"
          sizes="32x32"
          href="/favicon-32x32.png"
        />
        <link
          rel="icon"
          type="image/png"
          sizes="16x16"
          href="/favicon-16x16.png"
        />
        <link rel="manifest" href="/site.webmanifest" />
        <link rel="mask-icon" href="/mask-icon.svg" color="#111111" />
        <link
          rel="alternate"
          type="application/rss+xml"
          title="Blog posts"
          href="https://arne.me/blog/atom.xml"
        />
        <link
          rel="alternate"
          type="application/rss+xml"
          title="Arne’s  Weekly archive"
          href="https://arne.me/weekly/atom.xml"
        />
        <link href="mailto:hey@arne.me" rel="me authn" />
      </head>
      <body>
        <a href="#main" className="skip-link">
          Skip to content
        </a>
        <Header />
        <main id="main">{children}</main>
        <Footer />
      </body>
    </html>
  );
}
