// Builds the explorer's graph and table details from an RPFM schema.
//
// Patches follow the `Field` accessors in `rpfm_lib::schema`: a patch value replaces
// the field's own value for `is_key`, `is_reference`, `lookup`, `description`,
// `default_value`, `is_filename` and `filename_relative_path`. Other patch keys are
// kept as `patch_notes`.

(function (root, factory) {
    const api = factory();
    if (typeof module === "object" && module.exports) {
        module.exports = api;
    } else {
        root.RpfmSchemaModel = api;
    }
})(typeof self !== "undefined" ? self : this, () => {
    "use strict";

    const TABLES_SUFFIX = "_tables";
    const APPLIED_PATCH_KEYS = new Set([
        "is_key",
        "is_reference",
        "lookup",
        "description",
        "default_value",
        "is_filename",
        "filename_relative_path",
    ]);

    /**
     * Get a table's graph id: its name without the `_tables` suffix, as `is_reference` targets are stored.
     *
     * @param {string} tableName The table name, with or without the suffix.
     * @returns {string} The table id.
     */
    function tableId(tableName) {
        return tableName.endsWith(TABLES_SUFFIX) ? tableName.slice(0, -TABLES_SUFFIX.length) : tableName;
    }

    /**
     * Find the schema inside a parsed value, unwrapping the wrappers RPFM tools put around it.
     *
     * @param {*} value The parsed value.
     * @returns {{version: number, definitions: Object, patches: Object}} The schema.
     * @throws {Error} If no schema is found.
     */
    function normalizeSchema(value) {
        let current = value;
        for (let depth = 0; depth < 5 && current && typeof current === "object"; depth++) {
            if (current.definitions && typeof current.definitions === "object") {
                return {
                    version: current.version ?? null,
                    definitions: current.definitions,
                    patches: current.patches || {},
                };
            }

            const mcpText = current.result?.content?.[0]?.text;
            if (typeof mcpText === "string") {
                current = JSON.parse(mcpText);
            } else if (current.Schema) {
                current = current.Schema;
            } else {
                break;
            }
        }
        throw new Error("This file doesn't look like an RPFM schema: no 'definitions' found.");
    }

    function newestDefinition(definitions) {
        return definitions.reduce((newest, definition) => (definition.version > newest.version ? definition : newest));
    }

    function fieldTypeName(fieldType) {
        return typeof fieldType === "string" ? fieldType : Object.keys(fieldType || {})[0] || "Unknown";
    }

    function splitRelativePath(path) {
        return path.replace(/\\/g, "/").split(";");
    }

    /**
     * Get a field with its table's patches applied.
     *
     * @param {Object} field The field as stored in the schema.
     * @param {Object|undefined} tablePatches The patches of the field's table.
     * @returns {Object} The field, plus `field_type_name` and `patch_notes`.
     */
    function patchedField(field, tablePatches) {
        const patch = tablePatches?.[field.name] || {};
        const result = {
            name: field.name,
            field_type: field.field_type,
            field_type_name: fieldTypeName(field.field_type),
            is_key: field.is_key,
            default_value: field.default_value ?? null,
            is_filename: field.is_filename,
            filename_relative_path: field.filename_relative_path ? splitRelativePath(field.filename_relative_path) : null,
            is_reference: field.is_reference ?? null,
            lookup: field.lookup ?? null,
            description: field.description || "",
            ca_order: field.ca_order,
            is_bitwise: field.is_bitwise,
            enum_values: field.enum_values || {},
            is_part_of_colour: field.is_part_of_colour ?? null,
            patch_notes: {},
        };

        if ("is_key" in patch) {
            result.is_key = patch.is_key === "true";
        }
        if ("default_value" in patch) {
            result.default_value = patch.default_value;
        }
        if ("is_filename" in patch) {
            result.is_filename = patch.is_filename === "true";
        }
        if ("filename_relative_path" in patch) {
            result.filename_relative_path = splitRelativePath(patch.filename_relative_path);
        }
        if ("is_reference" in patch) {
            const separator = patch.is_reference.indexOf(";");
            if (separator !== -1) {
                result.is_reference = [patch.is_reference.slice(0, separator), patch.is_reference.slice(separator + 1)];
            }
        }
        if ("lookup" in patch) {
            result.lookup = patch.lookup.split(";");
        }
        if ("description" in patch) {
            result.description = patch.description;
        }

        for (const [key, value] of Object.entries(patch)) {
            if (!APPLIED_PATCH_KEYS.has(key)) {
                result.patch_notes[key] = value;
            }
        }
        return result;
    }

    /**
     * Read-only view of a schema with the graph and lookups the explorer needs.
     */
    class SchemaModel {

        /**
         * @param {{version: number, definitions: Object, patches: Object}} schema The schema to explore.
         */
        constructor(schema) {
            this.schema = schema;
            this.graphCache = null;
            this.referencedByIndex = null;
        }

        /**
         * Build the table graph: one node per table, one edge per reference in each table's newest definition.
         *
         * Referenced tables missing from the schema get a node with `missing: true`.
         *
         * @returns {{schema_version: number, nodes: Object[], edges: Object[]}} The graph.
         */
        graph() {
            if (this.graphCache) {
                return this.graphCache;
            }

            const nodes = new Map();
            const edges = [];
            for (const [tableName, definitions] of Object.entries(this.schema.definitions)) {
                if (!Array.isArray(definitions) || definitions.length === 0) {
                    continue;
                }

                const newest = newestDefinition(definitions);
                const tablePatches = this.schema.patches[tableName];
                const fields = (newest.fields || []).map((field) => patchedField(field, tablePatches));
                const id = tableId(tableName);
                nodes.set(id, {
                    id,
                    table: tableName,
                    versions: definitions.map((definition) => definition.version).sort((a, b) => b - a),
                    fields: fields.length,
                    keys: fields.filter((field) => field.is_key).map((field) => field.name),
                    localised: (newest.localised_fields || []).length > 0,
                    missing: false,
                });

                for (const field of fields) {
                    if (field.is_reference) {
                        edges.push({
                            from: id,
                            from_column: field.name,
                            to: field.is_reference[0],
                            to_column: field.is_reference[1],
                            lookup: Array.isArray(field.lookup) && field.lookup.length > 0,
                        });
                    }
                }
            }

            for (const edge of edges) {
                if (!nodes.has(edge.to)) {
                    nodes.set(edge.to, {
                        id: edge.to,
                        table: edge.to + TABLES_SUFFIX,
                        versions: [],
                        fields: 0,
                        keys: [],
                        localised: false,
                        missing: true,
                    });
                }
            }

            this.graphCache = {
                schema_version: this.schema.version,
                nodes: [...nodes.values()].sort((a, b) => a.id.localeCompare(b.id)),
                edges,
            };
            return this.graphCache;
        }

        /**
         * Get every definition of a table, and the columns of other tables referencing it.
         *
         * @param {string} tableName The table name, with or without the `_tables` suffix.
         * @returns {Object|null} The table details, or null if the table isn't in the schema or referenced by it.
         */
        tableDetails(tableName) {
            const id = tableId(tableName);
            const fullName = id + TABLES_SUFFIX;
            const definitions = this.schema.definitions[fullName] || [];
            const referencedBy = this.referencedBy(id);
            if (definitions.length === 0 && Object.keys(referencedBy).length === 0) {
                return null;
            }

            const tablePatches = this.schema.patches[fullName];
            return {
                id,
                table: fullName,
                missing: definitions.length === 0,
                definitions: [...definitions]
                    .sort((a, b) => b.version - a.version)
                    .map((definition) => ({
                        version: definition.version,
                        fields: (definition.fields || []).map((field) => patchedField(field, tablePatches)),
                        localised_fields: (definition.localised_fields || []).map((field) => patchedField(field, tablePatches)),
                    })),
                referenced_by: referencedBy,
            };
        }

        /**
         * Get the columns referencing a table, across all versions of all tables, like
         * `Schema::referencing_columns_for_table` in `rpfm_lib`.
         *
         * @param {string} id The table id (no `_tables` suffix).
         * @returns {Object} Map of referenced column -> referencing table -> sorted referencing columns.
         */
        referencedBy(id) {
            if (!this.referencedByIndex) {
                this.referencedByIndex = this.buildReferencedByIndex();
            }

            const byColumn = this.referencedByIndex.get(id);
            const result = {};
            if (!byColumn) {
                return result;
            }
            for (const [column, byTable] of byColumn) {
                result[column] = {};
                for (const [table, columns] of [...byTable].sort((a, b) => a[0].localeCompare(b[0]))) {
                    result[column][table] = [...columns].sort();
                }
            }
            return result;
        }

        buildReferencedByIndex() {
            const index = new Map();
            for (const [tableName, definitions] of Object.entries(this.schema.definitions)) {
                const tablePatches = this.schema.patches[tableName];
                for (const definition of definitions || []) {
                    for (const rawField of definition.fields || []) {
                        const field = patchedField(rawField, tablePatches);
                        if (!field.is_reference) {
                            continue;
                        }

                        const [targetId, targetColumn] = field.is_reference;
                        if (!index.has(targetId)) {
                            index.set(targetId, new Map());
                        }
                        const byColumn = index.get(targetId);
                        if (!byColumn.has(targetColumn)) {
                            byColumn.set(targetColumn, new Map());
                        }
                        const byTable = byColumn.get(targetColumn);
                        if (!byTable.has(tableName)) {
                            byTable.set(tableName, new Set());
                        }
                        byTable.get(tableName).add(field.name);
                    }
                }
            }
            return index;
        }
    }

    return { SchemaModel, normalizeSchema, patchedField, tableId };
});
