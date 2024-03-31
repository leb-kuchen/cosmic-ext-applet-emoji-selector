[cargo generate](https://cargo-generate.github.io/cargo-generate/)
# Example
```sh
cargo generate  "leb-kuchen/libcosmic-applet-template" -d config=true -d translate=true -d example=true  -d animation=true -d id="com.example.applet" -d icon="display-symbolic" --name="cosmic-applet-example"
cd cosmic-applet-example
# to install icons to system: cp icons... data/icons/scalable/apps/
cargo b -r
sudo just install
```
