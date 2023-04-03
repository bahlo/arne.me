import * as fs from "fs";
import * as https from "https";
import * as unzipper from "unzipper";

const zipUrl = process.env.FONT_ZIP_URL!;
const outputDir = "public/fonts";

https
  .get(zipUrl, async (response) => {
    if (response.statusCode !== 200) {
      throw new Error(`Failed to download zip: ${response.statusCode}`);
    }

    response.pipe(unzipper.Extract({ path: outputDir }));

    response.on("end", () => {
      console.log("Zip download and extraction complete.");
    });
  })
  .on("error", (error) => {
    console.error(`Failed to download zip: ${error.message}`);
  });
