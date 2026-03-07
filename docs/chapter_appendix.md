# Appendix

- Basic shortcuts (non-editable) for EVERY Table View provided by Qt:

![If I cut my arm, it'll be a shortcut or a longcut?](./images/image29.png)

- If RPFM crashes, it'll generate an error log in his folder called "error-report-xxxxxxx.toml". That file can help me find the problem, so if you want to help reporting the bug, send me that file too.

- **DON'T OPEN FILES WITH RPFM AND OTHER PROGRAMS LIKE PFM AND THE ASSEMBLY KIT AT THE SAME TIME**!!!!! Just in case you don't realise the problem, let me explain it: to not fill your entire RAM with data you probably aren't going to need, RPFM only reads from disk when needed and what it needs. This means that, if you open the same file with another program, that program **MAY LOCK YOUR FILE, CAUSING EITHER A CORRUPTED PACKFILE OR A VANISHED PACKFILE WHEN SAVING**.

- If you still want to do it, disable the `Use Lazy-Loading` Setting in the `Preferences` and the entire PackFile will be loaded to RAM. Weird things may still happen, but if the PackFile is loaded to RAM, you can just click `Save PackFile As...` and your PackFile will be saved properly.