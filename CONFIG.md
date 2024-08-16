# Config

## Types

### `SkinToneMode`

Skin tones are represented as a `uint32`.
If bits 8 to 28 are set, the skin tone is matched exactly.
Bits 28 and 29 only change the filter mode and do not correspond to a skin tone.
`NO_SKIN` is used to represent emojis without a skin tone, and `OTHER` is used for any future skin tones.
By default only the six least-significant bits are modified by the application.
The two most-significant bits must not be set.
However these bits could be used in the future.

```rs
const DEFAULT = 1;

const LIGHT = 1 << 1;
const MEDIUM_LIGHT = 1 << 2;
const MEDIUM = 1 << 3;
const MEDIUM_DARK = 1 << 4;
const DARK = 1 << 5;
const NO_SKIN = 1 << 6;

const OTHER = 1 << 7;

const LIGHT_AND_MEDIUM_LIGHT = 1 << 8;
const LIGHT_AND_MEDIUM = 1 << 9;
const LIGHT_AND_MEDIUM_DARK = 1 << 10;
const LIGHT_AND_DARK = 1 << 11;
const MEDIUM_LIGHT_AND_LIGHT = 1 << 12;
const MEDIUM_LIGHT_AND_MEDIUM = 1 << 13;
const MEDIUM_LIGHT_AND_MEDIUM_DARK = 1 << 14;
const MEDIUM_LIGHT_AND_DARK = 1 << 15;
const MEDIUM_AND_LIGHT = 1 << 16;
const MEDIUM_AND_MEDIUM_LIGHT = 1 << 17;
const MEDIUM_AND_MEDIUM_DARK = 1 << 18;
const MEDIUM_AND_DARK = 1 << 19;
const MEDIUM_DARK_AND_LIGHT = 1 << 20;
const MEDIUM_DARK_AND_MEDIUM_LIGHT = 1 << 21;
const MEDIUM_DARK_AND_MEDIUM = 1 << 22;
const MEDIUM_DARK_AND_DARK = 1 << 23;
const DARK_AND_LIGHT = 1 << 24;
const DARK_AND_MEDIUM_LIGHT = 1 << 25;
const DARK_AND_MEDIUM = 1 << 26;
const DARK_AND_MEDIUM_DARK = 1 << 27;

const ALL = !0 >> 5;

const FILTER_EXACT = 1 << 28;
const ALL_EXACT = ((1 << 21) - 1) << 8;
const FILTER_INTERSECT = 1 << 29;
```

### `ClickMode`

Represented as bitflags, which can be `NONE` | `COPY` | `CLOSE` `APPEND` | `PRIVATE`.

- `NONE`: No action is performed.
- `COPY`: Copies the emoji to the clipboard.
- `CLOSE`: Closes the popup.
- `APPEND`: Appends the emoji to the search input.
- `PRIVATE`: Prevents the emoji from being added to the history.


### `ColorButton`: `{color: Color, active: bool, skin_tone_mode: SkinToneMode}`
An button with a background of `color`, when pressed setting the bits of `Config.skin_tone_mode`.


### `Color`: `[float, float, float, float]`
Represents a color in the sRGB color space.
RGB colors can be converted to sRGB by dividing by 255.0
Each sRGB component must be in the range 0.0 to 1.0 inclusive.


## Fields

### `skin_tone_mode`: `SkinToneMode`
Filters emojis based on their skin tone.
The default is `NO_SKIN | DEFAULT`.

### `left_click_action`, `right_click_action`, `middle_click_action`: `ClickMode`
The action performed when clicking on an emoji with these respected mouse buttons.

### `font_family`: `string`
The font used to render emojis.

### `last_used`: `string[]`
History of the last copied emojis. 

### `last_used_limit`: `uint`
Limits the emojis history size.

### `show_preview`: `bool`
Whether to show a preview of the currently selected emoji.

### `color_buttons`: `ColorButton[]` 
``` ```
