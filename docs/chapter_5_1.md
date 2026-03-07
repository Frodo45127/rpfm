# DB Types

People have complained that the types used in the DB Decoder and, by extension, in the tables are ***not intuitive enough***, so here is a little explanation to shut you up:
- `Bool`: One byte. Can be 00 or 01.
- `Float`, or `f32`: 4 bytes that represent a floating point number. Can be really anything.
- `Integer`, or `i32`: 4 bytes that represent a signed integer (admits negative numbers). Can be really anything.
- `Long Integer` or `i64`: 8 bytes that represent a signed integer (admits negative numbers). Can be really anything.
- `StringU8`: An UTF-8 String. It has an u16 (2 bytes) at the begining that specify his lenght, and then the String itself with each character encoded in one byte.
- `StringU16`: An UTF-16 String. It has an u16 (2 bytes) at the begining that specify his lenght, and then the String itself with each character encoded in two bytes.
- `OptionalStringU8`: Like a UTF-8 String, but with a bool before. If the bool is true, there is a `StringU8` after it. If it's false, **then there is nothing more** of that field after it.
- `OptionalStringU16`: Like a UTF-16 String. but with a bool before. If the bool is true, there is a `StringU16` after it. If it's false, then **there is nothing more** of that field after it.
- `SequenceU32`: It's a table inside a table.

There are some extra types that RPFM doesn't yet support for one reason or another:
- `OptionalInteger` (This one may not exists): Like an Integer, but with a bool before. If the bool is true, there is a `Integer` after it. If it's false, then there is nothing more of that field after it. Only seen in one table in Warhammer 2.

If you need more help to understand these types, please search on google.
