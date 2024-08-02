rootdir := ''
prefix := '/usr'
# Installs files into the system
install: 
    install -Dm0755 ./target/release/cosmic-applet-emoji-selector  {{rootdir}}{{prefix}}/bin/cosmic-ext-applet-emoji-selector
    install -Dm0644 data/dev.dominiccgeh.CosmicAppletEmojiSelector.desktop {{rootdir}}{{prefix}}/share/applications/dev.dominiccgeh.CosmicAppletEmojiSelector.desktop
    find 'data'/'icons' -type f -exec echo {} \; | rev | cut -d'/' -f-3 | rev | xargs -d '\n' -I {} install -Dm0644 'data'/'icons'/{} {{rootdir}}{{prefix}}/share/icons/hicolor/{}


install-schema:
    mkdir -p ~/.config/cosmic/dev.dominiccgeh.CosmicAppletEmojiSelector/v1/
    cp  data/schema/* ~/.config/cosmic/dev.dominiccgeh.CosmicAppletEmojiSelector/v1/