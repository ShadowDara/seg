import { readFile, writeFile } from "node:fs/promises";
import { createInterface } from "node:readline/promises";
import { stdin as input, stdout as output } from "node:process";
import { parse, stringify } from "smol-toml";

const rl = createInterface({ input, output });

const version = await rl.question("Neue Version: ");

rl.close();

const packageJson = JSON.parse(
  await readFile("package.json", "utf8")
);

const cargo = parse(
  await readFile("Cargo.toml", "utf8")
);

const pyproj = parse(
  await readFile("pyproject.toml", "utf8")
);

packageJson.version = version;
cargo.package.version = version;
pyproj.project.version = version;


await writeFile(
  "package.json",
  JSON.stringify(packageJson, null, 2) + "\n"
);

await writeFile(
  "Cargo.toml",
  stringify(cargo)
);

await writeFile(
  "pyproject.toml",
  stringify(pyproj)
);

await writeFile(
  "seg/__init__.py",
  `__version__ = "${version}"`
)

console.log(`Version aktualisiert: ${version}`);
