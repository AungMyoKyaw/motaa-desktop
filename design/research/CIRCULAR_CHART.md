# Circular chart: evidence and corrected design

The supplied circular-chart screenshot matches chart **mode 9**, implemented by `main.m2274m0()` (original DEX name `m0`), rather than a generic Western aspect wheel. This correction replaces the first circular design's invented aspect chords.

## Verified from the extracted app

Research root: `<research-archive>/motaa-v31.1/`.

| Feature | Evidence in `decompiled/sources/motaa/myanmar/main.java` |
| --- | --- |
| Mode 9 calls `m2274m0`; mode 10 calls `m2259N0` | 20744–20781 |
| Base center (240,240), ring radii 240,220,145,125,95,78,60 | 29984–30003 |
| Profile date, time, place, zodiac, ayanamsha and house system in central disc | 30009–30029 |
| Ascendant index 0; Sun through Ketu indices 1–9; sign-relative degree formatting | 30031–30108 |
| 12 sign divisions; 27 nakshatra sections with names, ruler codes and numbers | 30134–30179 |
| 108 radial divisions, exact syllable labels and repeating quarter-code values | 30190–30243 |
| Colored house midpoint points, black cusp dots, two edges per midpoint | 30258–30396 |
| Planet positions quantized to 108 divisions; colliding markers move inward by 12 units | 30401–30435 |
| Coordinates: x=240+r*cos(longitude+75°), y=240−r*sin(longitude+75°) for normalized angles | 2666–2675 and 8037–8042 |
| Planet marker color classification table and palette | 2001–2010 and 2026–2174 |

`f4234h3` holds house cusps from `AbstractC0297z.m791g`; `f4242j3` holds adjacent-cusp circular midpoints from `m797m`. Assignments are at main.java:17120–17123. Helper evidence: `decompiled/sources/p023L0/AbstractC0297z.java:377–380`, `406–483`, and `724–802`.

JADX duplicates part of the colored-connector loop. The authoritative `decoded/smali/motaa/myanmar/main.smali:265194–265720` confirms one loop over houses 1–12, wrapping the last boundary to the first. Verified house colors are yellow, cyan, pale blue, green, pale gold, red, green, red, gold, green, lavender and red.

The earlier `N0` renderer has different intermediate radii (180,157,140) and additional arc-classification layers. Those were initially discovered during investigation, but **are not the geometry used for this mode-9 reconstruction**.

## Translation and notation

Navigation, controls, titles, central metadata and the ring guide are English. Nakshatra names use abbreviated English transliterations to fit the ring; full source names are retained in the data file. The 108 syllables are linguistic source data and remain in their original script. Per the user’s later preference, all numeric astrological codes now display as English digits (0–9). All 27 full display names match the user-provided contents image, with short ring labels supported by a complete 16 px name key.

`F0(7)` emits **ဝ** (U+101D), while ruler/code tables contain **၀** (U+1040). The raw evidence preserves the distinction. The display uses English 0 for the Saturn numeric code while leaving the linguistic syllable ဝ unchanged. The ruler sequence is literally ၉,၆,၁,၂,၃,၈,၅,၀,၄, repeated three times.

## Fixture and remaining uncertainty

The birth date/time and planet degree values come from the supplied demonstration. Absolute longitudes use the sign offsets corroborated by the demo's LatLon screen; Sun 0°34′ in its sign is 90°34′ absolute in this fixture. Equal-house cusp positions are an inferred fixture from the displayed Ascendant, not new astronomical calculations. Planet colors are evaluated from the original D0 table for those fixture positions.

The figure reconstructs source geometry, counts, connectivity, source tables and placement rules. It does **not** establish numerical parity of the ephemeris, timezone conversion, house calculations or calendar. `x0` uses rounded decimal thresholds; the static fixture avoids ambiguous boundary values and does not claim boundary-equivalent porting. The source's saturated data colors are preserved; this is not a claim of full accessibility compliance for every tiny data glyph.

## Artifacts

- `../source/circle-data.json`: source arrays, color tables, source hash and fixture provenance.
- `m0-renderer.java.txt`: bounded excerpt of the original extracted renderer.
- `../source/circular_chart.py`: source-informed SVG construction.
- `../previews/02-circular-chart.png`: normal workspace with detail entry point.
- `../previews/10-circular-detail.png`: larger 760-pixel chart with English ring guide.

These findings are static observations of the retained Java and Smali. No APK was executed during this investigation.
