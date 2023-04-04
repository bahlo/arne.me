export default function render(rating: number): string {
  let str = "";
  for (let i = 0; i < rating; i++) {
    str += "★";
  }
  for (let i = 0; i < 5 - rating; i++) {
    str += "☆";
  }

  return str;
}
