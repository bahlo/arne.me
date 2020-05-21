const Cache = require('@11ty/eleventy-cache-assets');

const USERNAME = process.env.FEEDBIN_USERNAME;
const PASSWORD = process.env.FEEDBIN_PASSWORD;

module.exports = Cache(`https://${USERNAME}:${PASSWORD}@api.feedbin.com/v2/subscriptions.json`, {
  duration: '1d',
  type: 'json',
});
