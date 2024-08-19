![screenshot-2024-03-31-08-18-52](https://github.com/leb-kuchen/cosmic-applet-emoji-selector/assets/102472435/496eae10-a889-46c4-b802-08c0aa4df078)

# Install 
```sh
git clone https://github.com/leb-kuchen/cosmic-ext-applet-emoji-selector 
cd cosmic-ext-applet-emoji-selector 
cargo b -r
sudo just install
```

# Config
The configuration directory is `~/.config/cosmic/dev.dominiccgeh.CosmicAppletEmojiSelector/v1/`.
In addition, the default schema can be installed with `just install-schema`

# Emoji font
`Noto Color Emoji` is the default emoji font and is required by default. 
The default can be changed in `~/.config/cosmic/dev.dominiccgeh.CosmicAppletEmojiSelector/v1/font_family`.
A font which supports Unicode 15.1 is generally recommended.

# License
Files without an SPDX identifier are licensed under the MIT LICENSE