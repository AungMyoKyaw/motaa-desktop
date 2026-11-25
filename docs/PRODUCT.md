# MOTAA Desktop

<!-- impeccable:product-schema 1 -->

## Platform

web

Desktop application delivered through Tauri. Desktop window conventions govern the shell.

## Stack

Tauri requested by the user. FigCraft CLI is the required first design tool. Frontend framework is delegated by the user's instruction to proceed.

## Product Purpose

Recreate the demonstrated MOTAA V31.1 horoscope workflows as a desktop application using the supplied reverse engineering and app screenshots.

## Users

Working assumption following the user's instruction to proceed: Burmese-speaking astrologers working with saved birth profiles, charts, aspects and periods.

## Capabilities and Constraints

The complete design covers all eleven recovered MOTAA modes: Rasi, Tricakka, traditional aspects, nakshatra traversal, LatLon, Western aspects, natal/transit comparison, Dasha, circular chart, Rahu and Rasi strengths. Planet and Ascendant epochs are independently selectable. Settings retain all nine ayanamshas, eight house systems, two zodiacs and two lunar-node choices.

The user approved including the advertised ZarTarPro capabilities as working features in the eventual desktop application. Their proposed workspaces cover strength timelines, Ashtakavarga, KP, heliocentric coordinates, divisional charts, interpretations, electional timing, ephemeris, eclipses, transit changes, ten pillars/Ayu Moon, nakshatra groups, daily Tatkalika, Satapati, Sarvatobhadra and yoga/candidate-time analysis. These advertised modules have design specifications; their formulas and interpretation sources require further research before implementation.

Approved desktop changes: selected-date-relative time scrubbing; explicit validation; ordinary names with spaces; unknown birth time; local access to every mode without membership activation; workspace calculation overrides with explicit profile saving; decimal and DMS coordinates; historical time zones with manual override; duplicate/undo/unsaved-change states; JSON backup/restore; four original theme families. Original-database migration requires a sample database and format validation.

Delivery order: complete the full FigCraft design, then implement in stages starting with the recovered MOTAA workflows. This repository currently contains the design and its generation tools, not a functional Tauri application. See design/IMPLEMENTATION_PLAN.md for staged engineering work and acceptance boundaries.

Confirmed interface language: English. Display all chart numbers with English digits (0–9). The circular chart’s original syllable ring remains linguistic data notation. Nakshatra display names follow the user-supplied contents image. Working defaults: local saved profiles and macOS development first. Additional operating-system release targets remain unconfirmed.

Arithmetic must be checked against the original application's outputs. Static reverse engineering and screenshots do not establish numerical parity. Demonstration values in the design are reference or illustrative values, never computed results.

## Brand Commitments

Preserve the MOTAA name, traditional chart geometry, Burmese notation and the recognizable cream/gold color family. Modernize the desktop composition.

## Evidence on Hand

- 29 timestamped PNG screenshots in the repository root.
- <research-archive>/motaa-v31.1/REPORT.md
- METHOD_CATALOG.md and UI_CATALOG.md in that research directory.
- Decompiled Java, Smali, resources and bundled Myanmar/astrology fonts.

## Product Principles

- Keep the active person's birth details and calculation settings visible.
- Preserve traditional notation while giving dense results room to be read.
- Keep time stepping available while exploring charts, aspects and periods.
- Separate recorded reference data from verified calculation output.

## Accessibility & Inclusion

Design for Myanmar script with adequate line height, readable table values, visible keyboard focus and textual labels alongside color distinctions.
