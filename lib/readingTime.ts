const wordsPerMinute = 183; // https://www.sciencedirect.com/science/article/abs/pii/S0749596X19300786?via%3Dihub

export default function calculateReadingTimeMinutes(text: string) {
  const wordCount = text
    .split(/\s+/)
    .filter((word) => word.match(`[a-zA-Z0-9]`)).length;
  return Math.ceil(wordCount / wordsPerMinute);
}
