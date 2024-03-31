# Installs files into the system
install: 
    sudo install -Dm0755 ./target/release/cosmic-applet-emoji-selector  /usr/bin/cosmic-applet-emoji-selector
    sudo install -Dm0644 data/dev.dominiccgeh.CosmicAppletEmojiSelector.desktop /usr/share/applications/dev.dominiccgeh.CosmicAppletEmojiSelector.desktop
    find 'data'/'icons' -type f -exec echo {} \; | rev | cut -d'/' -f-3 | rev | xargs -d '\n' -I {} sudo install -Dm0644 'data'/'icons'/{} /usr/share/icons/hicolor/{}