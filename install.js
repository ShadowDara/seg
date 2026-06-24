const fs = require("fs");
const path = require("path");
const https = require("https");
const { execSync } = require("child_process");

const version = require("./package.json").version;

const platformMap = {
  "linux-x64": "x86_64-unknown-linux-gnu.tar.gz",
  "darwin-x64": "x86_64-apple-darwin.tar.gz",
  "darwin-arm64": "aarch64-apple-darwin.tar.gz",
  "win32-x64": "x86_64-pc-windows-msvc.zip"
};

const key = `${process.platform}-${process.arch}`;

const asset = platformMap[key];

if (!asset) {
  console.error(`Unsupported platform: ${key}`);
  process.exit(1);
}

const url =
  `https://github.com/Shadowdara/seg/releases/download/v${version}/${asset}`;

fs.mkdirSync("bin", { recursive: true });

const archive =
  path.join("bin", asset);

const file = fs.createWriteStream(archive);

https.get(url, res => {
  res.pipe(file);

  file.on("finish", () => {
    file.close();

    if (asset.endsWith(".zip")) {
      execSync(
        `powershell Expand-Archive "${archive}" bin`,
        { stdio: "inherit" }
      );
    } else {
      execSync(
        `tar -xzf ${archive} -C bin`,
        { stdio: "inherit" }
      );
    }

    fs.unlinkSync(archive);

    console.log("Installed seg");
  });
});
