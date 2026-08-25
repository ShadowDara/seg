
type Config = Entry[];

type Entry = CustomEntry | string;

interface CustomEntry {
  type: string;
  object: string;
  file: string;
}

function loadconfig(): Config {
  
}

function main() {
  let config = loadconfig();

  for (const es in config) {
    if (es === "package.json") {
      // package json stuff

      continue
    }

    if (es === "Cargo.toml") {
      // cargo stuff

      continue
    }

    if (es === "pyproject.toml") {
      // pyprojec stuff

      continue
    }
  }
}