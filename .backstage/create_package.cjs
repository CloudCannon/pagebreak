const fs = require("fs");
const path = require("path");

const cwd = process.cwd();
const package_name = `@pagebreak/${path.basename(cwd)}`;
const [os, cpu] = process.argv.slice(2);

if (!os || !cpu) {
    console.error("Script os and cpu as positional arguments");
    process.exit(1);
}

fs.writeFileSync(path.join(cwd, "package.json"), JSON.stringify({
    name: package_name,
    version: "0.0.0",
    description: `The platform-specific binary for pagebreak on ${os}/${cpu}`,
    license: "MIT",
    repository: {
        type: "git",
        url: "git+https://github.com/cloudcannon/pagebreak.git"
    },
    author: "CloudCannon",
    os: [ os ],
    cpu: [ cpu ],
}));

fs.writeFileSync(path.join(cwd, "README.md"), `# Pagebreak

The platform-specific binary for pagebreak on ${os}/${cpu}
`);