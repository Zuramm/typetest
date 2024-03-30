# Typetest

Test your typing in the terminal. Heavily inspired by [Monkey Type](https://monkeytype.com/) and [Ngram Type](https://ranelpadon.github.io/ngram-type/). 
The program doesn't use a fullscreen UI like [TUI](https://docs.rs/tui/latest/tui/), but rather uses [console codes](https://man7.org/linux/man-pages/man4/console_codes.4.html) to overlap your typing with the test.
You have to supply the word sets via a pipe where each word is on a seperate line. 
There are two modes: random and permutation.
The random mode requires an amount of words and then picks randomly from the supplied word set.
The permutation mode changes the order of the input set. You can set how many words at once should in a test using the combination argument and repeat each test with the repetition argument.

For example if the file `english-200.txt` contains the 200 most common english words sorted by frequency. You can test using all 200 words like this:
```sh
cat engligh-200.txt | typetest random 20
```
Or if you only want to test using the first 50 words: (Note: This makes sense with sets sorted by frequency)
```sh
head -n 50 english-200.txt | typetest random 20
```

To test like on Ngram Type, you can use the permutation mode. Given a file containing 200 bigrams called `bigrams.txt`.
```sh
cat bigrams.txt | typetest permutations --combination 3 --repetition 2
# or
cat bigrams.txt | typetest permutations -c 3 -r 2
```

In addition you can set minimum wpm (speed), accuracy or consistency. For example
```sh
head -n 50 sets/bigrams | typetest --min-wpm 40 --min-accuracy 100 permutation -c 5 -r 2
```

> (!) Note: The program has a bug on windows.

## Download

You can find the precompiled linux, mac, and windows version in the repeases section.

## Install

You can install typetest by cloning it and running `cargo install`

```sh
git clone git@github.com:Zuramm/typetest.git
cargo install
```
