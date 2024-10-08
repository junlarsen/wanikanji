# WaniKani is cool, but its SRS is hardly configurable

This is a small program that queries WaniKani's API for kanji subjects and adds them to a local Anki deck using the Anki
Connect API.

This repository contains a toolsuite for exporting WaniKani's data to Anki.

## Prerequisites

I don't plan on publishing this project on crates.io, so you'll need to build and run it from source yourself.

- Rust 1.81 or newer (older versions may work, but I haven't tested them)
- Anki 2.1.x
- AnkiConnect Plugin for Anki 2.1.x (https://ankiweb.net/shared/info/2055492159)
- WaniKani Account with subscription
- WaniKani API Token (https://www.wanikani.com/settings/personal_access_tokens)

## Authentication

Because the subject (kanji & vocab) content is proprietary content of WaniKani, you'll have to authenticate and download
all the subject data yourself. To do this, you will need to create a new API token that you will pass to the program. It
will take care of downloading all the kanji and vocabulary data locally for you. This is done to cooperate with the
intellectual property rights and terms of WaniKani and its API usage.

## Commands

```text
Export your WaniKani data into Anki decks

Usage: wanikanji [OPTIONS] <COMMAND>

Commands:
  query-kanji             Download all kanji data from wanikani
  query-vocabulary        Download all vocabulary data from wanikani
  create-kanji-deck       Create Anki deck and Anki card type for Kanji
  create-vocabulary-deck  Create Anki deck and Anki card type for Vocabulary
  install-kanji           Install previously downloaded Kanji data into Anki deck
  install-vocabulary      Install previously downloaded Vocabulary data into Anki deck
  help                    Print this message or the help of the given subcommand(s)

Options:
      --cache-dir <CACHE_DIR>          [default: .cache]
      --api-token <API_TOKEN>
      --anki-endpoint <ANKI_ENDPOINT>  [default: http://localhost:8765]
  -h, --help                           Print help
  -V, --version                        Print version
```

Remember to always download the data (with query-kanji or query-vocabulary) before installing it into Anki, otherwise
you will receive an error.

## Other

**Why?**: I'm living in Japan for a year, and for my own interest I would like to out-pace the default timing of
WaniKani. As a user who was previously at level 52 before resetting my account, I would like to go a lot faster than
what WaniKani will allow me to do.

## License

This project is licensed under the [Mozilla Public License Version 2.0](LICENSE).
