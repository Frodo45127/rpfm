# Templates

Templates allows you to bootstrap mods for specific purpouses. The basic idea is this: you want to create a new building? Then just load the building template, fill the parameters you're asked for, hit Ok, and you will have a working mod. A new unit? Same, fill some fields, hit ok, done.

It has some advantages over templated packfiles:
- Templates always makes sure all generated tables are valid and up-to-date, so it can be used in tutorials without need for updating in each patch.
- Templates are a bit dynamic, allowing you to change on the fly many parameters that otherwise would require you to manually go into many tables after importing them.

## How to use them?
Just go to `About`/`Update Templates` and wait until it tells you you have the latest ones. Then, just go to `PackFile`/`Load Template` and click in the one you want to load. Fill the fields it asks you, then hit ok. That's all.

## How to make them?
Making them is a bit more complicated. First, where are they? They're stored in RPFM's Config folder. There should be two folders there:
- `templates`: templates downloaded from the official repo.
- `templates_custom`: templates made by yourself.

Inside of those folders there is a folder per game (templates are per-game), and inside each folder name there are two folders:
- `assets`: Binary files that will be loaded directly into RPFM when loading a template. For example, dummy tga files.
- `definitions`: Definition files for each template. These are simple JSON files. NOTE: There can be a conflict name between a custom definition and an official one, but the custom one will be the one used.

About how to write one..., it's hard to explain, so I've made an example. In the templates folder, you'll find:
- `schema.json`: Contains the definition of a template json. It can be used in a JSON validator to validate your templates.
- `warhammer_2/definitions/siege_battle.json`: Example definition you can use as a base for your own templates.

You just need to make a valid json for your template, put it in the definitions folder of the game you want, and it should appear in RPFM next time you change the game selected.

Keep in mind that the templates are community-driven, so if there is no template for what you want... feel free to make one yourself and do a PR on the template repo.
