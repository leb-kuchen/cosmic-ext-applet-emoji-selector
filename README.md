![emoji selector UI](https://github.com/leb-kuchen/cosmic-applet-emoji-selector/assets/102472435/496eae10-a889-46c4-b802-08c0aa4df078)
Panel applet for copying emoji to the clipboard

# Install 
```sh
git clone https://github.com/leb-kuchen/cosmic-ext-applet-emoji-selector 
cd cosmic-ext-applet-emoji-selector 
cargo b -r
sudo just install
```
after which you will be able to place this applet into your dock through COSMIC settings:

![COSMIC panel settings](https://github.com/user-attachments/assets/cc2323ec-6c5b-4172-96b8-9f00e35aee49)

# Usage
Click the smiley icon in the panel to bring up the selector. Browse or select your desired emoji,
then click it to copy the emoji to the clipboard.

# Config
The configuration directory is `~/.config/cosmic/dev.dominiccgeh.CosmicAppletEmojiSelector/v1/`.
In addition, the default schema can be installed with `just install-schema`

# Emoji font
`Noto Color Emoji` is the default emoji font and is required by default. 
The default can be changed in `~/.config/cosmic/dev.dominiccgeh.CosmicAppletEmojiSelector/v1/font_family`.
A font which supports Unicode 15.1 is generally recommended.

# License
Files without an SPDX identifier are licensed under the MIT LICENSE
