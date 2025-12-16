import { createServer, IncomingMessage, ServerResponse } from "node:http";
import { createReadStream, existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const PORT = parseInt(process.env.PORT || "8787", 10);
const RELEASES_DIR = join(__dirname, "../releases");
const RELEASES_JSON = join(RELEASES_DIR, "releases.json");

interface ReleaseConfig {
  version: string;
  signature: string;
  fileSize: number;
  filename: string;
}

interface ReleasesJson {
  [filename: string]: {
    version: string;
    signature: string;
    length: number;
  };
}

function loadReleasesConfig(): ReleasesJson {
  if (existsSync(RELEASES_JSON)) {
    try {
      return JSON.parse(readFileSync(RELEASES_JSON, "utf-8"));
    } catch {
      console.warn("Warning: Failed to parse releases.json");
    }
  }
  return {};
}

function findLatestRelease(): ReleaseConfig | null {
  if (!existsSync(RELEASES_DIR)) {
    return null;
  }

  const files = readdirSync(RELEASES_DIR).filter((f) => f.endsWith(".dmg"));
  if (files.length === 0) {
    return null;
  }

  // Sort by version (assuming filename format: tauri-app_{version}_aarch64.dmg)
  files.sort((a, b) => {
    const versionA = a.match(/_(\d+\.\d+\.\d+)_/)?.[1] || "0.0.0";
    const versionB = b.match(/_(\d+\.\d+\.\d+)_/)?.[1] || "0.0.0";
    return versionB.localeCompare(versionA, undefined, { numeric: true });
  });

  const latestFile = files[0];
  const version =
    latestFile.match(/_(\d+\.\d+\.\d+)_/)?.[1] ||
    process.env.UPDATE_VERSION ||
    "0.2.0";
  const filePath = join(RELEASES_DIR, latestFile);
  const stats = statSync(filePath);

  // Load signature from releases.json or environment variable
  const releasesConfig = loadReleasesConfig();
  const fileConfig = releasesConfig[latestFile];
  const signature = fileConfig?.signature || process.env.ED_SIGNATURE || "";

  return {
    version,
    signature,
    fileSize: stats.size,
    filename: latestFile,
  };
}

function generateAppcast(config: ReleaseConfig): string {
  const pubDate = new Date().toUTCString();

  return `<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <channel>
    <title>Tauri App Updates</title>
    <link>http://localhost:${PORT}/appcast.xml</link>
    <description>Appcast for Tauri App</description>
    <language>en</language>
    <item>
      <title>Version ${config.version}</title>
      <sparkle:version>${config.version}</sparkle:version>
      <sparkle:shortVersionString>${config.version}</sparkle:shortVersionString>
      <sparkle:minimumSystemVersion>11.0</sparkle:minimumSystemVersion>
      <pubDate>${pubDate}</pubDate>
      <enclosure
        url="http://localhost:${PORT}/releases/${config.filename}"
        sparkle:edSignature="${config.signature}"
        length="${config.fileSize}"
        type="application/octet-stream"
      />
      <description><![CDATA[
        <h2>What's New in ${config.version}</h2>
        <ul>
          <li>Test update release</li>
          <li>Bug fixes and improvements</li>
        </ul>
      ]]></description>
    </item>
  </channel>
</rss>`;
}

function generateNoUpdateAppcast(): string {
  return `<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle">
  <channel>
    <title>Tauri App Updates</title>
    <link>http://localhost:${PORT}/appcast.xml</link>
    <description>Appcast for Tauri App</description>
    <language>en</language>
  </channel>
</rss>`;
}

function handleRequest(req: IncomingMessage, res: ServerResponse): void {
  const url = req.url || "/";
  console.log(`${req.method} ${url}`);

  // Handle appcast.xml request
  if (url === "/appcast.xml") {
    const release = findLatestRelease();

    res.setHeader("Content-Type", "application/xml; charset=utf-8");

    if (release) {
      if (!release.signature) {
        console.warn(
          "Warning: ED_SIGNATURE environment variable not set. Update verification may fail."
        );
      }
      res.statusCode = 200;
      res.end(generateAppcast(release));
      console.log(`Serving appcast for version ${release.version}`);
    } else {
      res.statusCode = 200;
      res.end(generateNoUpdateAppcast());
      console.log("No releases found, serving empty appcast");
    }
    return;
  }

  // Handle DMG file download
  if (url.startsWith("/releases/")) {
    const filename = url.replace("/releases/", "");
    const filePath = join(RELEASES_DIR, filename);

    if (!existsSync(filePath)) {
      res.statusCode = 404;
      res.end("File not found");
      console.log(`File not found: ${filename}`);
      return;
    }

    const stats = statSync(filePath);
    res.setHeader("Content-Type", "application/octet-stream");
    res.setHeader("Content-Length", stats.size);
    res.setHeader(
      "Content-Disposition",
      `attachment; filename="${filename}"`
    );
    res.statusCode = 200;

    const stream = createReadStream(filePath);
    stream.pipe(res);
    console.log(`Serving file: ${filename} (${stats.size} bytes)`);
    return;
  }

  // Handle root and other paths
  if (url === "/") {
    const release = findLatestRelease();
    res.setHeader("Content-Type", "text/html; charset=utf-8");
    res.statusCode = 200;
    res.end(`
<!DOCTYPE html>
<html>
<head>
  <title>Appcast Server</title>
  <style>
    body { font-family: system-ui, sans-serif; max-width: 800px; margin: 2rem auto; padding: 0 1rem; background: #fff; color: #000; }
    code { background: #f4f4f4; padding: 0.2rem 0.4rem; border-radius: 4px; color: #000; }
    pre { background: #f4f4f4; padding: 1rem; border-radius: 8px; overflow-x: auto; color: #000; }
    a { color: #0066cc; }
    @media (prefers-color-scheme: dark) {
      body { background: #1e1e1e; color: #e0e0e0; }
      code { background: #2d2d2d; color: #e0e0e0; }
      pre { background: #2d2d2d; color: #e0e0e0; }
      a { color: #6bb3ff; }
    }
  </style>
</head>
<body>
  <h1>Appcast Server</h1>
  <p>Server is running on port <code>${PORT}</code></p>

  <h2>Endpoints</h2>
  <ul>
    <li><a href="/appcast.xml"><code>GET /appcast.xml</code></a> - Appcast XML feed</li>
    <li><code>GET /releases/:filename</code> - Download DMG files</li>
  </ul>

  <h2>Current Release</h2>
  ${
    release
      ? `
  <ul>
    <li><strong>Version:</strong> ${release.version}</li>
    <li><strong>File:</strong> ${release.filename}</li>
    <li><strong>Size:</strong> ${(release.fileSize / 1024 / 1024).toFixed(2)} MB</li>
    <li><strong>Signature:</strong> ${release.signature ? "Set" : '<span style="color:red">Not set (ED_SIGNATURE)</span>'}</li>
  </ul>
  `
      : '<p style="color:orange">No releases found in <code>releases/</code> directory</p>'
  }

  <h2>Usage</h2>
  <pre>
# Sign a DMG file
../../../sparkle-bin/sign_update releases/tauri-app_0.2.0_aarch64.dmg

# Start server with signature
ED_SIGNATURE="your-signature" pnpm dev
  </pre>
</body>
</html>
    `);
    return;
  }

  // 404 for other paths
  res.statusCode = 404;
  res.end("Not found");
}

const server = createServer(handleRequest);

server.listen(PORT, () => {
  console.log(`\nAppcast server running at http://localhost:${PORT}`);
  console.log(`\nEndpoints:`);
  console.log(`  GET /appcast.xml    - Appcast XML feed`);
  console.log(`  GET /releases/:file - Download DMG files`);
  console.log(`\nRelease directory: ${RELEASES_DIR}`);

  const release = findLatestRelease();
  if (release) {
    console.log(`\nLatest release: v${release.version} (${release.filename})`);
    console.log(`  Size: ${(release.fileSize / 1024 / 1024).toFixed(2)} MB`);
    console.log(`  Signature: ${release.signature ? "Configured" : "Not set"}`);
    if (!release.signature) {
      console.warn(
        `\nWarning: Signature not configured. Add to releases/releases.json or set ED_SIGNATURE env var.`
      );
    }
  } else {
    console.log(`\nNo releases found. Add DMG files to releases/ directory.`);
  }
});
