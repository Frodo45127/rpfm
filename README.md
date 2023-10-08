![rpfm](https://github.com/Frodo45127/rpfm/assets/15714929/3b0ffaaf-e29a-4df9-b537-59a4612a88cc)
# Rusted PackFile Manager
***Rusted PackFile Manager*** (RPFM) is a... reimplementation in Rust and ***~~GTK3~~ Qt5*** of ***PackFile Manager*** (PFM), one of the best modding tools for Total War Games. Not only it can edit Packs, but also has integrated editors for DB Tables, Loc files, scripts,... and a bunch of different file formats.

**Downloads here:** [https://github.com/Frodo45127/rpfm/releases][Downloads]

**Manual here:** [***HERE, READ IT BEFORE ASKING***][Manual].

[![become_a_patron_button](https://user-images.githubusercontent.com/15714929/40394531-2130b9ce-5e24-11e8-91a2-bbf8e6e75d21.png)][Patreon]

# Requirements (to use)
* ***Windows***: Just download it, extract it somewhere and execute it.
* ***Linux***:
    - ***Arch Linux and derivates***: it's in the AUR as **rpfm-bin**.
    - ***Other distros***: Make sure you have Qt5 5.14 or higher, xz, and 7zip installed. DDS files also require you to have the Qt5 Imageformats DDS library installed.
* ***MacOS***: You'll know it when I manage to compile it for Mac.

# Requirements (to build)

Check the building instructions [here][CompInst]

# FAQ
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

[Rustup download]: https://www.rustup.rs/ "Here you can download it :)"
[Patreon]: https://www.patreon.com/RPFM
[Manual]: https://frodo45127.github.io/rpfm/
[Downloads]: https://github.com/Frodo45127/rpfm/releases
[CompInst]: https://frodo45127.github.io/rpfm/chapter_comp.html
