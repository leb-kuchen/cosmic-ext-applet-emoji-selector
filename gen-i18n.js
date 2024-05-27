
import { Glob } from "bun"
import { parseArgs } from "util";
import { dirname, join } from "path"
import { $ } from "bun"


const containsEmoji = require('contains-emoji');

// maybe some type checking
const { positionals: [path, path2, outDir,] } = parseArgs({
    allowPositionals: true,

})
const localeRe = new RegExp(`/(?:annotations|annotationsDerived)/([^/]*)/?`)


const pathsProm = Array.fromAsync(new Glob(path).scan())
const paths2Prom = Array.fromAsync(new Glob(path2).scan())
let paths = await Promise.all([pathsProm, paths2Prom])
paths = map(paths, (pathPair, i) => map(pathPair, path => ({ path, full: i === 0 })))






const pathIt = map(flatten(paths), pathObj => ({ locale: getLocale(pathObj.path), ...pathObj }))
const pathMap = Map.groupBy(pathIt, ({ locale }) => locale)






for (const [locale, localPathObj] of pathMap) {

    if (localPathObj.length != 2) {
        console.error(localPathObj)
        continue

    }
    if (!localPathObj[0].full || localPathObj[1].full) {
        throw Error("unexpected order")
    }
    const [annotations1, annotations2] = await Promise.all(map(localPathObj, ({ path }) => Bun.file(path).json()))

    const emojis1 = annotations1?.annotations?.annotations
    const emojis2 = annotations2?.annotationsDerived?.annotations

    if (!emojis1 || !emojis2) {
        console.error(`${path} did not contain required emojis: ${locale}`)
        continue
    }


    const emojis = { ...emojis2, ...emojis1 }

    console.log(locale, emojis["😀"])

    // maybe filter json directly and convert fluent files to json and vice versa

    let translationFile = ""
    for (const [emoji, data] of Object.entries(emojis)) {
        // todo generate own emoji dict, dont trust this guy
        if (!containsEmoji(emoji)) {
            continue
        }

        const addTranslation = (name, term) => {
            if (!name) {
                // console.error(`${path} - ${name}(${term}) : ${emoji} is null or empty`);
                return
            }
            const emoji_txt = Array.from(emoji, c => c.codePointAt().toString(16)).join("-")
            translationFile += `${term}-${emoji_txt} = ${name}\n`
        };

        addTranslation(data?.default?.at?.(0), "default")
        addTranslation(data?.tts?.at(0), "tts")

    }

    const filepath = join(outDir, locale, "cosmic_applet_emoji_selector.ftl")
    const dir = dirname(filepath)
    await $`mkdir -p ${dir}`
    await Bun.write(filepath, translationFile)

}







function getLocale(s) {
    const locale = localeRe.exec(s)[1]
    return locale

}

function* map(it, f) {
    let count = 0
    for (const v of it) {
        yield f(v, count++)
    }
}
function* flatten(it) {
    for (const v of it) {
        yield* v
    }
}
