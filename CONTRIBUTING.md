### Rules for contributing to RPFM.
You can contribute to RPFM development in multiple ways:
- **Docs**: There is a manual, but tutorials are always welcome.
- **Schemas**: If you want to update an schema, just send me the edited file and I'll add your changes (or edit it and do a pull request, whatever you want).
- **Code**: If you want to contribute with some code change, there are two rules: ***explain what your code does*** and ***no black magic code***. The reason for those rules is I'm using this project to learn Rust, so complex code without explanation doesn't help me learn anything.

Also, ***no rustfmt here***. Many of the changes that it does mess up the current format (I tested it and the code grew up **from 18k lines** of rust code to **more than 30k**). So no rustfmt here. Clippy-suggested fixes are welcome though.

If you don't have any idea why something is done, just ask. Even I forgot why some things are done when I don't write a comment about it.
