# cal

Display a calendar.

## Implemented

- Single month display (default)
- Three-month display (`-3`)
- Year display (`-y`)
- Next twelve months (`-Y`)
- N-month display (`-n`)
- Monday (`-m`) or Sunday (`-s`) as first day of week
- Day-of-year numbering (`-j`)
- Configurable number of columns (`-c`)
- Positional arguments: `[[[day] month] year]`
- Month names and abbreviations as arguments (e.g., `cal March`)
- Special date words: `today`, `tomorrow`, `yesterday`

## Not yet implemented

- `--vertical` (ncal-style vertical layout)
- `--reform` (handling of the Julian to Gregorian calendar reform date)
- `--week` (ISO/US week number display)
- `--color` (terminal colorization of today, weekends, headers)
- `--span` (center multi-month display on the target date)
- Month and day names from the locale (currently English only)
- Relative date expressions (`+2days`, `1 week ago`)
