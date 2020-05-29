# Initial Configuration

![Preferences... everyone has them.](./images/image2.png)

After we start RPFM for the first time, we have to configure a couple of things.

First, it should promt you to update your schemas. Hit yes and, if it fails, go to `About/Check Schema Update` and keep hitting update untils it succeeds.

Then, we need to go to `PackFile/Preferences`, and the window above this will popup. It seems like a lot of new stuff to know, but it's really simple. First the paths:
- `MyMod's folder`: it's the path where your ***MyMod*** will be stored. ***MyMod*** are explained in a later chapter of this documentation, so for now you just need to know that it's a path RPFM will use to store stuff for your mods. Set it pointing to an empty folder.
- `XXX folder`: These are the folders where your games are. Not where /data is, but **WHERE THE .EXE IS!!!** RPFM uses them for plenty of things, so remember to set them for the games you have.

In the end, it should look something like this:

![Paths... all of them end in Rome.](./images/image3.png)

Next, the `Default Game`. RPFM uses a `Game Selected` setting to configure certain parts of the program to work with one game or another. For example, it changes the way the mods are saved, the default folder to save them, **the schema used for the tables**,.... Here you can set the game that'll be selected by default when you open the program.

Next, all those checkboxes. You can get an explanation about what they do just by hovering them with the mouse, like this.

![Hovering before it was cool!.](./images/image4.png)

There are a couple of settings that may need some aditional explanation:
- `Use Dark Theme`: Self-explanatory, but only available in Windows. The Linux version **uses the system's Qt Theme** instead.
- **The entire debug section**: Stuff that an user will never need, but it helps me a lot. Don't use them unless you really need it.

And finally, the `Shortcuts` button. Hitting it will open the `Shortcuts` window, where you can see and edit all the shortcuts currently used by RPFM.

![Shortcuts are little cuts....](./images/image5.png)

Just keep in mind that some of **the shortcuts are applied when the program starts**, so you'll have to close and re-open RPFM for the changes to take effect.

When you're done with the settings, just hit `Save`. You can restore them to the defaults with the button of the left (same for the shortcuts with their `Restore Defaults` button). One thing to take into account is that, if any of the paths is invalid, RPFM will delete it when hitting `Save`.

Now the last step. This is optional, but recommendable and it **requires you to have the Assembly Kit** for your games installed. We have to go to `Special Stuff` and, for each game we have, hit `Generate PAK File`. This will create a special file that will help RPFM with reference data for table columns. It's **not enabled for Empire and Napoleon** for now, but it should work for every other game.

With that, we have completed the initial configuration. So now that we're done configuring RPFM, let's take a look at the features it has to offer.
