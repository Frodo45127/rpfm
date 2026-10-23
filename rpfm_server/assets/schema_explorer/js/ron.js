// Minimal RON (Rusty Object Notation) parser, enough to read RPFM schema files in the browser.
//
// Values come out in the same shape serde_json gives for the same Rust data, so the rest of the
// explorer works the same whether it loads a `.ron` schema or a `.json` one:
// structs become objects, tuples and lists arrays, maps objects, `Some(x)` x, `None` null,
// unit enum variants strings, and enum variants with data `{ "Variant": data }`.

(function (root, factory) {
    const api = factory();
    if (typeof module === "object" && module.exports) {
        module.exports = api;
    } else {
        root.RpfmRon = api;
    }
})(typeof self !== "undefined" ? self : this, () => {
    "use strict";

    const CHAR_TAB = 9;
    const CHAR_LF = 10;
    const CHAR_CR = 13;
    const CHAR_SPACE = 32;
    const CHAR_QUOTE = 34;
    const CHAR_SLASH = 47;
    const CHAR_STAR = 42;
    const CHAR_BACKSLASH = 92;

    const IDENTIFIER = /[A-Za-z_][A-Za-z0-9_]*/y;
    const NUMBER = /[+-]?(?:0x[0-9A-Fa-f_]+|0b[01_]+|0o[0-7_]+|(?:[0-9][0-9_]*)?\.?[0-9][0-9_]*(?:[eE][+-]?[0-9_]+)?)/y;

    class RonError extends Error {
        constructor(message, text, position) {
            const before = text.slice(0, position);
            const line = before.split("\n").length;
            const column = position - before.lastIndexOf("\n");
            super(`${message} (line ${line}, column ${column})`);
            this.name = "RonError";
            this.position = position;
        }
    }

    class Parser {
        constructor(text) {
            this.text = text;
            this.pos = 0;
        }

        fail(message) {
            throw new RonError(message, this.text, this.pos);
        }

        skipWhitespace() {
            const text = this.text;
            for (;;) {
                const code = text.charCodeAt(this.pos);
                if (code === CHAR_SPACE || code === CHAR_LF || code === CHAR_CR || code === CHAR_TAB) {
                    this.pos++;
                } else if (code === CHAR_SLASH && text.charCodeAt(this.pos + 1) === CHAR_SLASH) {
                    const end = text.indexOf("\n", this.pos);
                    this.pos = end === -1 ? text.length : end + 1;
                } else if (code === CHAR_SLASH && text.charCodeAt(this.pos + 1) === CHAR_STAR) {
                    const end = text.indexOf("*/", this.pos + 2);
                    if (end === -1) {
                        this.fail("Unterminated block comment");
                    }
                    this.pos = end + 2;
                } else {
                    return;
                }
            }
        }

        peek() {
            return this.text[this.pos];
        }

        expect(char) {
            this.skipWhitespace();
            if (this.text[this.pos] !== char) {
                this.fail(`Expected '${char}'`);
            }
            this.pos++;
        }

        // Consumes `char` if it's next, so callers can handle optional trailing commas.
        accept(char) {
            this.skipWhitespace();
            if (this.text[this.pos] === char) {
                this.pos++;
                return true;
            }
            return false;
        }

        value() {
            this.skipWhitespace();
            const char = this.peek();
            switch (char) {
                case "\"": return this.string();
                case "(": return this.structOrTuple();
                case "[": return this.list();
                case "{": return this.map();
                case "'": return this.char();
                case undefined: return this.fail("Unexpected end of input");
                default: break;
            }

            if (char === "r" && (this.text[this.pos + 1] === "\"" || this.text[this.pos + 1] === "#")) {
                return this.rawString();
            }

            const number = this.match(NUMBER);
            if (number !== null) {
                return parseNumber(number);
            }

            const identifier = this.match(IDENTIFIER);
            if (identifier === null) {
                return this.fail(`Unexpected character '${char}'`);
            }
            return this.identifierValue(identifier);
        }

        match(regex) {
            regex.lastIndex = this.pos;
            const found = regex.exec(this.text);
            if (found === null) {
                return null;
            }
            this.pos = regex.lastIndex;
            return found[0];
        }

        identifierValue(identifier) {
            switch (identifier) {
                case "true": return true;
                case "false": return false;
                case "None": return null;
                case "inf": return Infinity;
                case "NaN": return NaN;
                case "Some": {
                    this.expect("(");
                    const inner = this.value();
                    this.accept(",");
                    this.expect(")");
                    return inner;
                }
                default: break;
            }

            this.skipWhitespace();
            if (this.peek() !== "(") {
                return identifier;
            }

            // Enum variant with data. A single tuple item is a newtype variant, like serde_json.
            const data = this.structOrTuple();
            const payload = Array.isArray(data) && data.length === 1 ? data[0] : data;
            return { [identifier]: payload };
        }

        structOrTuple() {
            this.expect("(");
            if (this.accept(")")) {
                return [];
            }

            return this.isStructField() ? this.structBody() : this.tupleBody();
        }

        // A struct starts with `identifier:` (but not `identifier::`), anything else is a tuple.
        isStructField() {
            const start = this.pos;
            const identifier = this.match(IDENTIFIER);
            this.skipWhitespace();
            const isField = identifier !== null && this.peek() === ":" && this.text[this.pos + 1] !== ":";
            this.pos = start;
            return isField;
        }

        structBody() {
            const object = {};
            do {
                this.skipWhitespace();
                if (this.peek() === ")") {
                    break;
                }
                const name = this.match(IDENTIFIER);
                if (name === null) {
                    this.fail("Expected a field name");
                }
                this.expect(":");
                object[name] = this.value();
            } while (this.accept(","));
            this.expect(")");
            return object;
        }

        tupleBody() {
            const items = [];
            do {
                this.skipWhitespace();
                if (this.peek() === ")") {
                    break;
                }
                items.push(this.value());
            } while (this.accept(","));
            this.expect(")");
            return items;
        }

        list() {
            this.expect("[");
            const items = [];
            do {
                this.skipWhitespace();
                if (this.peek() === "]") {
                    break;
                }
                items.push(this.value());
            } while (this.accept(","));
            this.expect("]");
            return items;
        }

        // Map keys become strings, as JSON object keys do.
        map() {
            this.expect("{");
            const object = {};
            do {
                this.skipWhitespace();
                if (this.peek() === "}") {
                    break;
                }
                const key = this.value();
                this.expect(":");
                object[String(key)] = this.value();
            } while (this.accept(","));
            this.expect("}");
            return object;
        }

        string() {
            const text = this.text;
            this.pos++;
            let result = "";
            let chunkStart = this.pos;
            for (;;) {
                const code = text.charCodeAt(this.pos);
                if (Number.isNaN(code)) {
                    this.fail("Unterminated string");
                }
                if (code === CHAR_QUOTE) {
                    result += text.slice(chunkStart, this.pos);
                    this.pos++;
                    return result;
                }
                if (code === CHAR_BACKSLASH) {
                    result += text.slice(chunkStart, this.pos);
                    result += this.escape();
                    chunkStart = this.pos;
                } else {
                    this.pos++;
                }
            }
        }

        char() {
            this.pos++;
            const value = this.text[this.pos] === "\\" ? this.escape() : this.text[this.pos++];
            if (this.text[this.pos] !== "'") {
                this.fail("Unterminated char");
            }
            this.pos++;
            return value;
        }

        escape() {
            const text = this.text;
            const kind = text[this.pos + 1];
            this.pos += 2;
            switch (kind) {
                case "\"": return "\"";
                case "'": return "'";
                case "\\": return "\\";
                case "/": return "/";
                case "n": return "\n";
                case "r": return "\r";
                case "t": return "\t";
                case "0": return "\0";
                case "b": return "\b";
                case "f": return "\f";
                case "x": {
                    const hex = text.slice(this.pos, this.pos + 2);
                    this.pos += 2;
                    return String.fromCharCode(parseInt(hex, 16));
                }
                case "u": {
                    if (text[this.pos] === "{") {
                        const end = text.indexOf("}", this.pos);
                        const hex = text.slice(this.pos + 1, end);
                        this.pos = end + 1;
                        return String.fromCodePoint(parseInt(hex, 16));
                    }
                    const hex = text.slice(this.pos, this.pos + 4);
                    this.pos += 4;
                    return String.fromCharCode(parseInt(hex, 16));
                }
                default:
                    this.pos -= 2;
                    return this.fail(`Unknown escape '\\${kind}'`);
            }
        }

        // Raw strings: `r"..."` or `r#"..."#` with any number of hashes.
        rawString() {
            this.pos++;
            let hashes = 0;
            while (this.text[this.pos] === "#") {
                hashes++;
                this.pos++;
            }
            if (this.text[this.pos] !== "\"") {
                this.fail("Expected '\"' in raw string");
            }
            const terminator = "\"" + "#".repeat(hashes);
            const end = this.text.indexOf(terminator, this.pos + 1);
            if (end === -1) {
                this.fail("Unterminated raw string");
            }
            const value = this.text.slice(this.pos + 1, end);
            this.pos = end + terminator.length;
            return value;
        }
    }

    function parseNumber(literal) {
        const clean = literal.replace(/_/g, "");
        const sign = clean.startsWith("-") ? -1 : 1;
        const unsigned = clean.replace(/^[+-]/, "");
        if (/^0[xbo]/.test(unsigned)) {
            const radix = { x: 16, b: 2, o: 8 }[unsigned[1]];
            return sign * parseInt(unsigned.slice(2), radix);
        }
        return Number(clean);
    }

    /**
     * Parse a RON document.
     *
     * @param {string} text The RON text.
     * @returns {*} The parsed value, shaped like serde_json would shape it.
     * @throws {RonError} If the text isn't valid RON.
     */
    function parse(text) {
        const parser = new Parser(text);
        const value = parser.value();
        parser.skipWhitespace();
        if (parser.pos < text.length) {
            parser.fail("Unexpected trailing data");
        }
        return value;
    }

    return { parse, RonError };
});
