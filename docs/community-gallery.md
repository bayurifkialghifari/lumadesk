# Community Gallery

The Community Gallery is a searchable catalog of animations that can be added directly to the local Collection. It uses a versioned static catalog and does not require an account or a dedicated LumaDesk server.

## Browse and import

Open **Gallery** from the navigation rail. Search by title, description, author, or tag, then narrow the results by format or tag. Every card displays its author, source, license, format, download size, and animated preview.

Select **Add to collection** to download and import an item. LumaDesk verifies the exact byte length and SHA-256 before the file enters the managed library. The catalog title and tags are saved with a new item. If the same content hash already exists, the card changes to **In collection** and no duplicate row or file is created.

## Offline behavior

LumaDesk checks a static JSON catalog hosted with the project. A valid catalog is cached for six hours. When the network is unavailable or a remote catalog is invalid, the application uses the last valid cached catalog. If no cache exists, it loads the starter catalog embedded in the application.

The starter previews and media are also embedded, so their preview and import paths work on first launch without network access. Remote entries added in later catalog revisions require network access until their files have been cached.

## Import from an HTTPS URL

Select **Import URL** to download one GIF, APNG, WebP, PNG, or JPEG file. Enter an HTTPS address and an optional display name. The app follows a bounded number of redirects, requires HTTPS at the destination, and rejects downloads larger than 100 MiB.

The temporary download is passed to the normal import pipeline. LumaDesk inspects the media, creates its content hash, checks for duplicates, copies valid content into managed storage, and removes the temporary file.

## Catalog trust and licensing

The catalog is data, not executable code. Rust validates the schema and limits before the UI receives it. Every asset includes a source URL, license name, license URL, exact size, and SHA-256. Source and license links open in the system browser so users can review them before import.

The bundled starter pack is released under CC0 1.0. Other entries may use a different redistributable license when their attribution is complete. Users remain responsible for following an item's license when redistributing it.

## Contribute an item

1. Add an original or redistributable asset under `src/assets/gallery/`.
2. Add an entry to `gallery/catalog.v1.json` with a stable lowercase kebab-case ID, author, source, license, tags, HTTPS URLs, byte size, and SHA-256.
3. Increase the catalog revision.
4. Run `pnpm validate:gallery`.
5. Open a pull request containing the asset, catalog update, and any required license text.

The validator rejects unknown fields, duplicate IDs, unsupported formats, insecure URLs, missing files, and incorrect integrity metadata.
