# Vendored libraries

Unmodified builds from the npm registry, pinned so the explorer works offline and loads no third-party code at runtime. Each tarball was checked against the registry's published `sha512` integrity value before extracting.

| File | Package | Version | License | SHA-256 of the file |
|---|---|---|---|---|
| `cytoscape.min.js` | [cytoscape](https://www.npmjs.com/package/cytoscape) (`dist/cytoscape.min.js`) | 3.30.4 | MIT, `cytoscape.LICENSE` | `1bb5340e549511e111b31e5684872c949ad33d40ea5dba0ad8e7d90c62c7b3b9` |
| `layout-base.js` | [layout-base](https://www.npmjs.com/package/layout-base) | 2.0.1 | MIT, `layout-base.LICENSE` | `ec15ab5df9af3f20708f4faab994accf91cda71848cd5bb10a23432cc50b6745` |
| `cose-base.js` | [cose-base](https://www.npmjs.com/package/cose-base) | 2.2.0 | MIT, `cose-base.LICENSE` | `7cae9509bd36235a63a85e71c8d9fa2cd0bc1d0c1ecc5b5a737976f39d040ddf` |
| `cytoscape-fcose.js` | [cytoscape-fcose](https://www.npmjs.com/package/cytoscape-fcose) | 2.2.0 | MIT, `cytoscape-fcose.LICENSE` | `4b1cab218d74996aa59cd8473f9239cc6398b8c1774d84d7e59ad9a68959cb57` |

`cytoscape-fcose` needs `cose-base`, which needs `layout-base`. They're UMD builds loaded as plain scripts (in that order), which expose the `cytoscape`, `layoutBase`, `coseBase` and `cytoscapeFcose` globals.

To update one, download the new tarball from `https://registry.npmjs.org/<package>/-/<package>-<version>.tgz`, verify it against `npm view <package>@<version> dist.integrity`, copy the same file and license over, and update this table (`sha256sum *.js`).
