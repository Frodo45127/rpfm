# `Special Stuff` Menu

![Because you are S.P.E.C.I.A.L!](./images/image10.png)

This menu contains... special features implemented for specific games. Basically, any feature that **doesn't really fit in any other place** goes here. Here we have:
- `Patch SiegeAI`: used in Warhammer 1 & 2 for **creating siege maps that the AI can handle**. Basically, make your map with the stuff required for the AI to work, and then patch his PackFile with this.
- `Optimize PackFile`: reduces the size of your PackFile and increase its compatibility with other mods by *cleaning* certain stuff on your packfile:
    - **DB**: Removes unchanged rows from vanilla. If table is empty, it removes it. Ignore files called the same as the vanilla ones (unless you disable that in the settings).
    - **Loc**: Removes unchanged rows from vanilla.
    - **Xml**: Removes xml files under the `terrain/tiles` folder, as those are leftovers of Terry's exports..
- `Generate Dependencies Cache`: generates a cache used for things like dependency checking, diagnostics, .... Doesn't work for Empire and Napoleon, yet.

There's also a `Rescue PackFile` feature that you SHOULD NOT USE UNLESS INSTRUCTED.
