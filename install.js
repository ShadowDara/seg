const fs = require("fs");
const path = require("path");
const https = require("https");
const { execSync } = require("child_process");

const version = require("./package.json").version;

const platformMap = {
  "linux-x64": "x86_64-unknown-linux-gnu.tar.gz",
  "darwin-x64": "x86_64-apple-darwin.tar.gz",
  "darwin-arm64": "aarch64-apple-darwin.tar.gz",
  "win32-x64": "x86_64-pc-windows-msvc.zip",
};

const key = `${process.platform}-${process.arch}`;
const asset = platformMap[key];

if (!asset) {
  console.error(`Unsupported platform: ${key}`);
  process.exit(1);
}

const url = `https://github.com/Shadowdara/seg/releases/download/v${version}/${asset}`;

fs.mkdirSync("bin", { recursive: true });

const archive = path.join("bin", asset);

function download(url, destination) {
  return new Promise((resolve, reject) => {
    https.get(url, (res) => {
      // Handle redirects
      if (
        res.statusCode === 301 ||
        res.statusCode === 302 ||
        res.statusCode === 307 ||
        res.statusCode === 308
      ) {
        return resolve(download(res.headers.location, destination));
      }

      if (res.statusCode !== 200) {
        return reject(
          new Error(`Download failed: ${res.statusCode} ${url}`)
        );
      }

      const file = fs.createWriteStream(destination);

      res.pipe(file);

      file.on("finish", () => {
        file.close(resolve);
      });

      file.on("error", reject);
    }).on("error", reject);
  });
}

(async () => {
  try {
    console.log(`Downloading ${url}`);

    await download(url, archive);

    const size = fs.statSync(archive).size;

    if (size < 1000) {
      throw new Error(
        `Downloaded file suspiciously small (${size} bytes)`
      );
    }

    console.log(`Downloaded ${size} bytes`);

    if (asset.endsWith(".zip")) {
      execSync(
        `powershell Expand-Archive "${archive}" bin`,
        { stdio: "inherit" }
      );
    } else {
      execSync(
        `tar -xzf "${archive}" -C bin`,
        { stdio: "inherit" }
      );
    }

    fs.unlinkSync(archive);

    console.log("Installed seg");
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
})();
