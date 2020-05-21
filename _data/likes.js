const Cache = require('@11ty/eleventy-cache-assets');

const USERNAME = process.env.PINBOARD_USERNAME;
const PASSWORD = process.env.PINBOARD_PASSWORD;

module.exports = Cache(`https://${USERNAME}:${PASSWORD}@api.pinboard.in/v1/posts/all?format=json&results=20&tag=like`, {
  duration: '1h',
  type: 'json',
});
