# DB Decoder

![Decoding your life, one step at a time.](./images/image23.png)

RPFM has an integrated **DB decoder**, to speed up a lot the decoding process of the **definition** of a table. It can be opened by right-clicking on a table file and selecting `Open/Open with Decoder`. Only works on tables.

The decoder screen is a bit complex so, like Jack the Ripper, let's check it in parts, one at a time. Starting by the left we have this:

![Raw data... like raw meat, but less toxic.](./images/image24.png)

This is the `PackedFile's Data` view. It's similar to a hexadecimal editor, but far less powerful, and it's not editable. In the middle you have **Raw Hexadecimal Data**, and in the right, you have a **Decoded version** of that data. To make it easier to work with it, both scrolling and selection are synchronised between both views. So you can select a byte in the middle view, and it'll get selected in the right one too. The colour code here means:

- **Red** : header of the table. It contains certain info about what's in the table, like his uuid, amount of rows,....
- **Yellow** : the part of the table already decoded following the structure from the fields table.
- **Magenta** : the byte where the next field after all the fields from the fields table starts.

Next, to the right, we have this:

![Fields.... like normal ones, but with less cows.](./images/image25.png)

This is the `Fields List`. Here are all the columns this table has, including their title, type, if they are a `key` column, their relation with other tables/columns, the decoded data on each field of the first row of the table, and a *Description* field, to add commentaries that'll show up when hovering the header of that column with the mouse.

If we right-click in any field of the table, we have these three self-explanatory options to help us with the decoding:

![Moving up and down.....](./images/image26.png)

And finally, under the `Fields List`, we have this:

![The guts of the decoder, exposed.](./images/image27.png)

The `Current Field Decoded` will show up the field that starts in the magenta byte of the `PackedFile's Data` view, decoded in the different types the tables use. It's use is simple: check what type makes more sense (for example, in the screenshot, it's evidently a `StringU8`), and click the `Use this` button in his row. Doing that will add a field of that type to the `Fields List`, and it'll update the `PackedFile's Data` View to show where the next field starts. Keep doing that until you think you've decoded the complete first row of the table, hit `Finish It!` at the right bottom corner, and select the table again. If the decoding is correct, the table will open. And that's how <s>I met your mother</s> you decode a table.

Under `Current Field Decoded` we have `Selected Field Decoded`. It does the same that `Current Field Decoded`, but from the byte you selected in the `PackedFile's Data` View. Just select a byte and it'll try to decode any possible field starting from it. It's for helping decoding complex tables.

To the right, we have some information about the table, and the `Versions List` (a list of versions of that table we have a definition for). If we right-click in one of them, we can load that version (useful to have something to start when a table gets *updated* in a patch) or delete it (in case we make a totally disaster and don't want it to be in the schema).

In the information of the table, the version number is only editable for version 0 tables. Usually, RPFM treats all versions as unique per-game, but version 0 really means ***no version***, so in older games, like empire, there can be multiple "version 0" tables with different definitions. For that, when a version 0 table is decoded, you can set its version to be negative, which will act as an alternative definition for that version 0 table.

It's only for Empire/Napoleon. Don't use it in recent games.

![Because no more is needed.](./images/image28.png)

And at the bottom, we have:
- `Import from Assembly Kit`: it tries to import the definition for this table from the assembly kit files. It tries.
- `Test Definition`: test the definition to see if it can decode the table correctly. If it fails, it'll show a json version of the rows of the table that it could decode.
- `Remove all fields`: removes all decoded fields, returning the table to a clean state.
- `Finish It!`: Save the `Fields List` as a new definition for that version of the table in the schema. The definition is inmediatly available after that, so the changes can be used immediately.
