/* eslint-disable @next/next/no-head-element */

import "../styles/global.css";

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html>
      <head></head>
      <body>
        <a href="#main" className="skip-link">Skip to content</a>
        <main id="main">
          {children}
        </main>
      </body>
    </html>
  );
}
