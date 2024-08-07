![screenshot-2024-03-31-08-18-52](https://github.com/leb-kuchen/cosmic-applet-emoji-selector/assets/102472435/496eae10-a889-46c4-b802-08c0aa4df078)

# Install 
```sh
git clone https://github.com/leb-kuchen/emoji-selector-applet-for-cosmic
cd emoji-selector-applet-for-cosmic
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

# Copying emojis
To copy emojis, `data_control` has to be enabled, to do set `COSMIC_DATA_CONTROL_ENABLED=1` in your profile.
Note this grants windowless applications access to your clipboard. This is related to applets not being able to interact
with the clipboard API (https://github.com/pop-os/cosmic-panel/issues/232). 

In case this does not meet your security requirements, you can enter the unicode code points manually.
To do so, enable `show_tooltip` and `show_unicode` in `~/.config/cosmic/dev.dominiccgeh.CosmicAppletEmojiSelector/v1/show_tooltip` and
`~/.config/cosmic/dev.dominiccgeh.CosmicAppletEmojiSelector/v1/show_unicode` respectivly.
Now press `Shift` + `Ctrl` + `U`, then enter the first code, e.g. `1F1E9`, finally press `Shift` + `Ctrl` to enter the code point. 
After that repeat this step for the remaining code points ( up to 8), in this example `1F1EA` and it will output 🇩🇪. 

