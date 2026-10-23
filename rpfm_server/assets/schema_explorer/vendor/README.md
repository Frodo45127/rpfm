# Vendored libraries

Unmodified builds from the npm registry, pinned so the explorer works offline and loads no third-party code at runtime. Each tarball was checked against the registry's published `sha512` integrity value before extracting.

| File | Package | Version | License | SHA-256 of the file |
|---|---|---|---|---|
| `cytoscape.min.js` | [cytoscape](https://www.npmjs.com/package/cytoscape) (`dist/cytoscape.min.js`) | 3.30.4 | MIT, `cytoscape.LICENSE` | `1bb5340e549511e111b31e5684872c949ad33d40ea5dba0ad8e7d90c62c7b3b9` |
| `graphology.umd.min.js` | [graphology](https://www.npmjs.com/package/graphology) (`dist/graphology.umd.min.js`) | 0.26.0 | MIT, `graphology.LICENSE` | `dc337efa23903f61e064c8e7e7f93a429e6855dccfc2458802b4ed30c621c087` |
| `graphology-library.min.js` | [graphology-library](https://www.npmjs.com/package/graphology-library) (`dist/graphology-library.min.js`) | 0.8.0 | MIT, `graphology-library.LICENSE` | `5323729ec6d4ac5d98a245873646c27fd80c7234df13972faec5869f3c54a105` |

They're UMD builds loaded as plain scripts, exposing the `cytoscape`, `graphology` and `graphologyLibrary` globals.

- Cytoscape.js draws the graph.
- graphology holds the graph for the layout.
- `graphologyLibrary.layoutForceAtlas2` (graphology-layout-forceatlas2 0.10) computes it.

The library bundle touches `DOMParser`, `XMLSerializer` and `Document` when it loads (for its GEXF/GraphML parsers). That's fine in browsers. The Node tests stub them.

To update one, download the new tarball from `https://registry.npmjs.org/<package>/-/<package>-<version>.tgz`, verify it against `npm view <package>@<version> dist.integrity`, copy the same file and license over, and update this table (`sha256sum *.js`).
