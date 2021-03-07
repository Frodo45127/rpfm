![rpfm](https://user-images.githubusercontent.com/15714929/90964588-3d36a080-e4c2-11ea-90fe-03168986b41a.png)
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
- ***Frodo45127***: I'm the guy who made the program.
- ***Maruka*** (From Steam): He made the wazard hat's icon. Also, he helped with the research to decode RigidModel files.
- ***Mr. Jox*** (From Steam): He helped with A LOT of information about decoding RigidModels.
- ***Aexrael Dex*** (From Discord): He is who got all those suggested functions you see when editing a Lua Script.
- ***DrunkFlamingo*** (From Discord): He is who got all the Lua Types for Warhammer 2 so Kailua can work with WH2 scripts.
- ***Der Spaten*** (From Discord): He helped with the research to decode RigidModels, specially with the "Replace texture" functionality.
- ***Trolldemorted*** (From Github): He is who made all the research and code to get Arena PackFiles (and music PackedFiles in Rome 2 and Attila) decrypted and readable.
- ***Jake Armitage*** (From Discord): He made the icons used by RPFM since version 2.0.0.
- ***John Sirett*** (From Gitlab): He made the original version of the CA_VP8 converter in python, which I used as a base for the converter in RPFM.
- ***Marthenil*** (From Discord): He managed to get most of the AnimXXX files decoded, and give some sense to what I saw when doing the same.

#### Special Thanks to:
- ***The guys that made PFM***: Most of the decoding stuff would have been very hard to work out without the PFM source code. So many thanks for make the PFM open source!
- ***The guys at CA***: They make good, easily-modable games, and are very friendly with the community. Weird company in these times.

[Rustup download]: https://www.rustup.rs/ "Here you can download it :)"
[Patreon]: https://www.patreon.com/RPFM
[Manual]: https://frodo45127.github.io/rpfm/
[Downloads]: https://github.com/Frodo45127/rpfm/releases
[CompInst]: https://frodo45127.github.io/rpfm/chapter_comp.html
