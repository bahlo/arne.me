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
      <head></head>
      <body>
        <a href="#main" className="skip-link">Skip to content</a>
        <Header />
        <main id="main">
          {children}
        </main>
        <Footer/>
      </body>
    </html>
  );
}
