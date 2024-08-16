rootdir := ''
prefix := '/usr'
sharedir :=  rootdir + prefix + '/share'
id := 'dev.dominiccgeh.CosmicAppletEmojiSelector'

# compressed: 7.3 mib, uncompressed: 94.4 mib, minified: 69.1 mib
# compressed and uncompressed files can also be mixed for different locales

compress := '0'
features := if compress == "1" { "--features compress" } else { "" }

build-release *args:
    cargo build --release {{features}} {{args}} 

# Installs files into the system
install: 
    install -Dm0755 ./target/release/cosmic-applet-emoji-selector  {{rootdir}}{{prefix}}/bin/cosmic-ext-applet-emoji-selector
    install -Dm0644 data/{{id}}.desktop {{rootdir}}{{prefix}}/share/applications/{{id}}.desktop
    find 'data'/'icons' -type f -exec echo {} \; | rev | cut -d'/' -f-3 | rev | xargs -d '\n' -I {} install -Dm0644 'data'/'icons'/{} {{rootdir}}{{prefix}}/share/icons/hicolor/{}
    for locale in `ls i18n-json`; do \
        file="i18n-json/$locale/annotations.json" ;\
        if [ {{compress}} -eq 1 ] ; then \
            xz -9c $file > "$file.xz" ;\
            file="$file.xz" ;\
        fi ;\
        install -Dm0644 -D -t "{{sharedir}}/{{id}}/i18n-json/$locale" $file ;\
    done


install-schema:
    mkdir -p ~/.config/cosmic/{{id}}/v1/
    cp  data/schema/* ~/.config/cosmic/{{id}}/v1/