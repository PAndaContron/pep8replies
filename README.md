# pep8replies

A Discord bot to check Python code blocks for PEP8 style issues, inspired by [pep8speaks](https://github.com/OrkoHunter/pep8speaks).

I've had an interesting experience with pep8speaks.
In principle, it's obviously a good thing to proactively avoid style issues in your code.
That being said, opening a pull request and immediately seeing a comment from pep8speaks about the 30 mistakes you made is more rage inducing than expected after the 15th time.
At some points, I've strongly wished I could strangle pep8speaks,
only to remember in despair that it is not a person but an abstract concept which I cannot harm in any way.

So, naturally, the only other option was to make something much worse.

pep8replies takes the idea behind pep8speaks and extends it to Discord rather than GitHub.
On Discord, you can post code blocks, and set a syntax highlighting language for each code block.
pep8replies will reply to any message with a Python highlighted codeblock telling you what PEP 8 style errors you made.
If you made none, pep8replies will simply congratulate you on your expertise.
This guarantees that you are *never* exempt from the expectation of writing clean, readable, clear, and concise code,
even in a message to your friends on Discord.

## Setup

You will need to have `flake8` installed by default for this bot to work;
to use a different linter, you can change the command in `config.json`.

The only necessary configuration is creating a `config.json` file in the root directory (at the same level as `src`, not inside it),
and putting the bot token in the `token` field.
You can also set `cmd` to an array of strings to be used for the linter command (each string is an argument).
The linter command is expected to read a codeblock from stdin, and output any style issues to stdout;
no output is considered a sign that there are no style issues.
The default command is `flake8 --stdin-display-name block -`.
Once everything is configured, you can run the bot with `cargo run --release`.
