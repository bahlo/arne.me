export default function SubscribeForm() {
  return (
    <>
      <h2 id="subscribe">Subscribe</h2>
      <p>
        Get the newsletter in your inbox every Sunday. No ads, no shenanigans.
      </p>
      <div className="weekly-subscription">
        <form
          action="https://buttondown.email/api/emails/embed-subscribe/arnesweekly"
          method="post"
        >
          <input
            required
            type="email"
            name="email"
            id="email"
            placeholder="you@example.org"
          />
          <input type="submit" value="Subscribe" />
        </form>
        <small>
          Your email address will be sent to{" "}
          <a href="https://buttondown.email">Buttondown</a>, the service I use
          to send out emails.
        </small>
      </div>
    </>
  );
}
