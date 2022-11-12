import Link from "next/link";

export default function Footer() {
  const now = new Date();
  return (
    <footer>
      <ul>
        <li>&copy; {now.getFullYear()} Arne Bahlo</li>
        <li>
          <Link href="/now">Now</Link>
        </li>
        <li>
          <Link href="/colophon">Colophon</Link>
        </li>
        <li>
          <Link href="/accessbility">A11y</Link>
        </li>
        <li>
          <Link href="/imprint">Imprint</Link>
        </li>
      </ul>
      <a rel="me" href="https://spezi.social/@arne" hidden></a>
    </footer>
  );
}