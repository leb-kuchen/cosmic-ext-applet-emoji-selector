rootdir := ''
prefix := '/usr'
sharedir :=  rootdir + prefix + '/share'
id := 'dev.dominiccgeh.CosmicAppletEmojiSelector'

build-release:
    cargo b -r

# Installs files into the system
install: 
    install -Dm0755 ./target/release/cosmic-applet-emoji-selector  {{rootdir}}{{prefix}}/bin/cosmic-ext-applet-emoji-selector
    install -Dm0644 data/{{id}}.desktop {{rootdir}}{{prefix}}/share/applications/{{id}}.desktop
    find 'data'/'icons' -type f -exec echo {} \; | rev | cut -d'/' -f-3 | rev | xargs -d '\n' -I {} install -Dm0644 'data'/'icons'/{} {{rootdir}}{{prefix}}/share/icons/hicolor/{}
    for locale in `ls i18n-json`; do \
        install -Dm0644 -D -t "{{sharedir}}/{{id}}/i18n-json/$locale" "i18n-json/$locale/annotations.json" ;\
    done


install-schema:
    mkdir -p ~/.config/cosmic/{{id}}/v1/
    cp  data/schema/* ~/.config/cosmic/{{id}}/v1/