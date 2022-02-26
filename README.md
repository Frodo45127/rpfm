![rpfm](https://user-images.githubusercontent.com/15714929/155857058-5e53c208-04fb-4557-a125-1054e1006ed9.png)
# Rusted PackFile Manager
***Rusted PackFile Manager*** (RPFM) is a... reimplementation in Rust and ***~~GTK3~~ Qt5*** of ***PackFile Manager*** (PFM), one of the best modding tools for Total War Games.

**Downloads here:** [https://github.com/Frodo45127/rpfm/releases][Downloads]

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

# Status of the Project
RPFM currently supports **all Total War Games since Empire Total War**. Improvements are done every week, and contributions, either by schema update, translation, code or docs are welcome!

# Requirements (to use)
* ***Windows***: Just download it, extract it somewhere and execute it.
* ***Linux***: Make sure you have Qt5 5.14 or higher, xz, and 7zip installed. DDS files also require you to have the Qt5 Imageformats DDS library installed.
* ***MacOS***: You'll know it when I manage to compile it for Mac.

Also, the manual is [***HERE, READ IT BEFORE ASKING***][Manual].

# Requirements (to build)

Check the building instructions [here][CompInst]

# FAQ
- **Why not helping with PFM instead of reimplementing it?**: because I wanted to learn a new language, and I already now a bit of C#. Also, where is the fun of that?
- **Why the code quality is not the very best?**: because I'm using this project to learn Rust, and I'm constantly rewriting code as I find new and better ways to write it.
- **How can I translate it to my own language?**: go to the locale folder, copy the *English_en.ftl* file, rename it to *NameYouWantInTheUI_xx.ftl*. For example, for spanish it'll be *Español_es.ftl*. Translate it. Done.
- **Why there is no .exe in the download?**: because you downloaded the source code, not the program. Check at the begining of this description, where is says ***Downloads here***.

# Credits
* Created and Programmed by: **Frodo45127**.
* Extra programming work by: **Vandy**.
* Modern DDS Read support by: **Phazer**.
* App Icons until v1.6.2 by: **Maruka**.
* App Icons since v2.0.0 by: **Jake Armitage**.
* AnimPack research: **Marthenil** and **Frodo45127**.
* Ca\_vp8 research: **John Sirett**.
* LUA functions by: **Aexrael Dex**.
* LUA Types for Kailua: **DrunkFlamingo**.
* RigidModel research by: **Mr.Jox**, **Der Spaten**, **Maruka**, **phazer** and **Frodo45127**.
* RigidModel module until v1.6.2 by: **Frodo45127**.
* RigidModel module since v2.4.99 by: **Phazer**.
* TW: Arena research and coding: **Trolldemorted**.
* TreeView Icons made by [Smashicons](\"https://www.flaticon.com/authors/smashicons\" "\"Smashicons\"") from [www.flaticon.com](\"https://www.flaticon.com/\" "\"Flaticon\""). Licensed under [CC 3.0 BY](\"http://creativecommons.org/licenses/by/3.0/\" "\"Creative")

[Rustup download]: https://www.rustup.rs/ "Here you can download it :)"
[Patreon]: https://www.patreon.com/RPFM
[Manual]: https://frodo45127.github.io/rpfm/
[Downloads]: https://github.com/Frodo45127/rpfm/releases
[CompInst]: https://frodo45127.github.io/rpfm/chapter_comp.html
