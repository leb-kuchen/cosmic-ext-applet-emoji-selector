
import { Glob } from "bun"
import { parseArgs } from "util";
import { mkdir, exists } from "node:fs/promises";
import { dirname, join } from "path"


const containsEmoji = require('contains-emoji');

const { positionals: [path, outDir,] } = parseArgs({
    allowPositionals: true,

})


const glob = new Glob(path)



for await (const path of glob.scan()) {

    const annotations = await Bun.file(path).json()

    const emojis = annotations?.annotations?.annotations
    if (!emojis) {
        console.error(`${path} did not contain required emojis`)
        continue
    }


    // maybe filter json directly and convert fluent files to json and vice versa

    let translationFile = ""
    for (const [emoji, data] of Object.entries(emojis)) {
        // todo generate own emoji dict, dont trust this guy
        if (!containsEmoji(emoji)) {
            console.log(emoji)
            continue
        }

        const addTranslation = (name, term) => {
            if (!name) {
                console.error(`${path} - ${name}(${term}) : ${emoji} is null or empty`);
                return
            }
            translationFile += `${term}-${emoji} = ${name}\n`
        };

        addTranslation(data?.default?.at?.(0), "default")
        addTranslation(data?.tts?.at(0), "tts")

    }
    const locale = path.slice(0, path.indexOf("/"))
    const filepath = join(outDir, locale, "cosmic_applet_emoji_selector.ftl")
    const dir = dirname(filepath)
    if (!await exists(dir)) {
        await mkdir(dir, { recursive: true, })

    }
    await Bun.write(filepath, translationFile)

}



