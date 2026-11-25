"""MOTAA FigCraft-only regression suite. The native master is the source of truth."""
from __future__ import annotations

import json
import struct
import sys
import unittest
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "design/source"))
from figcraft_master import MASTER, extract_screens, load_document, screen_layers  # noqa: E402
from export_tokens import generate  # noqa: E402



class FigCraftMasterTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.pages = extract_screens()
        cls.document = load_document()
        cls.nodes = cls.document["nodes"]
        cls.names = {screen["name"] for screen in cls.pages}

    def test_exactly_one_master(self):
        found = sorted(p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*.figcraft"))
        self.assertEqual(found, ["design/motaa-desktop.figcraft"])
        self.assertTrue(MASTER.is_file())

    def test_111_screens_and_preserved_original_11_modes(self):
        self.assertEqual(len(self.pages), 111)
        self.assertEqual(len(self.names), 111)
        originals = [
            "01 Natal chart", "12 Tricakka", "03 Aspects", "14 Nakshatra traversal",
            "15 LatLon", "16 Western aspects", "17 Natal transit", "04 Periods",
            "02 Circular chart", "18 Rahu wheel", "64 Rasi aspects mode",
        ]
        self.assertTrue(set(originals) <= self.names)
        self.assertEqual(len(self.document["nodes"]), 27397)

    def test_included_features_and_settings_source_flows(self):
        required = {
            "59 All features included", "60 Included feature overview", "61 No account required",
            "62 Offline feature access", "63 Rasi aspects ready",
            "65 Zodiac picker", "66 Ayanamsha picker", "67 House system picker",
            "68 Lunar node picker", "69 Birth coordinates", "70 Current coordinates",
            "72 Date and time editor", "73 Source time slider",
        }
        self.assertTrue(required <= self.names)
        visible = [node.get("characters", "") for node in self.nodes.values()
                   if node.get("type") == "text"]
        joined = " | ".join(visible)
        self.assertIn("Fagan–Bradley", joined)
        self.assertIn("JN Bhasin", joined)
        self.assertIn("Vehlow_Placidus", joined)
        self.assertIn("Vehlow_Equal", joined)
        self.assertIn("The midpoint is the current system time.", joined)
        self.assertIn("Calendar.getInstance()", joined)

    def test_navigation_and_native_variable_actions(self):
        navigate = 0
        set_variable = 0
        privacy_controls = 0
        ids = {node["id"] for node in self.nodes.values()}
        variables = {v["id"] for v in self.document["variables"]["variables"]}
        frameids = set()
        for page_id in self.document["pages"]:
            page = self.nodes[str(page_id)]
            self.assertEqual(len(page["children"]), 1)
            frameids.add(page["children"][0])
        for node in self.nodes.values():
            for interaction in node.get("interactions", []):
                for action in interaction.get("actions", []):
                    if action["type"] == "navigate":
                        self.assertIn(action["dest"], frameids)
                        navigate += 1
                    elif action["type"] == "setVariable":
                        self.assertIn(action["variable"], variables)
                        set_variable += 1
                    elif action["type"] == "conditional":
                        self.assertIn(node["name"], {"Birth details visibility button", "Birth details visibility label"})
                        self.assertEqual(len(action["blocks"]), 2)
                        for branch in action["blocks"]:
                            self.assertTrue(all(step["type"] == "setVariable" and step["variable"] in variables
                                                for step in branch["actions"]))
                        if self.nodes[str(node["parent"])]["type"] == "instance":
                            privacy_controls += 1
                    else:
                        self.fail(f'Unexpected action type {action["type"]}')
        self.assertEqual(navigate, 2456)
        self.assertEqual(set_variable, 71)
        self.assertEqual(privacy_controls, 110)
        self.assertEqual(sum(len(screen["links"]) for screen in self.pages), 2375)

    def test_walkthrough_links_follow_visible_controls(self):
        screens = {screen["name"]: screen for screen in self.pages}
        links = lambda name: screens[name]["links"]
        self.assertFalse(any(link["label"] == "Undo deletion"
                             for link in links("31 Profile library")))
        self.assertTrue(any(link["destination"] == "37 Backup restore"
                            and link["label"] == "Backup & restore"
                            for link in links("31 Profile library")))
        self.assertFalse(any(link["destination"] == "06 Birth profile" and
                             link["y"] in (100, 628)
                             for link in links("105 First launch - anonymous sample")))
        reached, pending = set(), ["01 Natal chart"]
        while pending:
            page = pending.pop()
            if page in reached:
                continue
            reached.add(page)
            pending.extend(link["destination"] for link in links(page))
        self.assertEqual(set(screens) - reached, {
            "05 Saved profiles", "08 Empty library", "105 First launch - anonymous sample",
        })

    def test_candidate_times_follow_birth_details_privacy(self):
        expected = {
            10364: ("Time window hidden · 5-minute interval", "12 Mar 1990 · 10:12–10:32 · 5-minute interval"),
            10372: ("Hidden", "10:17:00"),
            10380: ("Hidden", "12:00:00"),
            10387: ("Hidden", "10:27:00"),
        }
        variables = {item["id"]: item for item in self.document["variables"]["variables"]}
        for node_id, (hidden, shown) in expected.items():
            node = self.nodes[str(node_id)]
            variable = node["bound"]["characters"]
            self.assertEqual(node["characters"], hidden)
            self.assertEqual({entry["value"] for entry in variables[variable]["values"].values()}, {hidden})
            for control in self.nodes.values():
                if control["name"] not in {"Birth details visibility button", "Birth details visibility label"}:
                    continue
                for interaction in control.get("interactions", []):
                    blocks = interaction["actions"][0]["blocks"]
                    self.assertEqual(
                        [[step["value"] for step in block["actions"] if step["variable"] == variable]
                         for block in blocks],
                        [[hidden], [shown]],
                    )

    def test_reused_components_styles_and_four_theme_modes(self):
        types = Counter(node["type"] for node in self.nodes.values())
        self.assertEqual(types["component"], 296)
        self.assertEqual(types["componentSet"], 38)
        self.assertEqual(types["instance"], 1222)
        self.assertEqual(len(self.document["styles"]), 31)
        self.assertEqual(len(self.document["variables"]["variables"]), 134)
        collections = {
            collection["name"]: collection
            for collection in self.document["variables"]["collections"]
        }
        self.assertEqual(set(collections), {"MOTAA", "Gold Heritage / UI", "Birth details privacy", "MOTAA / Primitives"})
        self.assertEqual(
            [m["name"] for m in collections["MOTAA"]["modes"]],
            ["Light", "Pink", "Blue", "Monochrome"],
        )
        self.assertEqual(
            [m["name"] for m in collections["Gold Heritage / UI"]["modes"]],
            ["Light"],
        )

    def test_gold_heritage_v3_palette_and_chart_isolation(self):
        """Current native UI tokens and four source chart modes stay independent."""
        collections = {c["name"]: c for c in self.document["variables"]["collections"]}
        ui = collections["Gold Heritage / UI"]
        chart = collections["MOTAA"]
        self.assertEqual([m["name"] for m in ui["modes"]], ["Light"])
        self.assertEqual([m["name"] for m in chart["modes"]],
                         ["Light", "Pink", "Blue", "Monochrome"])
        colors = {
            x["name"]: x for x in self.document["variables"]["variables"]
            if x["collection"] == ui["id"]
        }
        self.assertEqual(len(colors), 49)
        variables = {x["id"]: x for x in self.document["variables"]["variables"]}
        collections_by_id = {c["id"]: c for c in self.document["variables"]["collections"]}
        def hexcolor(name):
            variable = colors[name]
            mode_id = ui["modes"][0]["id"]
            seen = set()
            while True:
                self.assertNotIn(variable["id"], seen)
                seen.add(variable["id"])
                value = variable["values"][mode_id]
                if value["type"] != "alias":
                    rgba = value["value"]
                    return "#" + "".join(f"{round(rgba[ch] * 255):02X}" for ch in "rgb")
                variable = variables[value["value"]]
                target_modes = collections_by_id[variable["collection"]]["modes"]
                if mode_id not in variable["values"]:
                    self.assertEqual(len(target_modes), 1)
                    mode_id = target_modes[0]["id"]
        self.assertEqual(hexcolor("ui/window"), "#F8F5EE")
        self.assertEqual(hexcolor("ui/sidebar"), "#F1ECDF")
        self.assertEqual(hexcolor("ui/action"), "#7C5B21")
        self.assertEqual(hexcolor("ui/brand"), "#8B6B2E")
        self.assertEqual(hexcolor("ui/selected"), "#EDE0BF")
        self.assertEqual(hexcolor("ui/control-border"), "#8E7F69")
        for name in ("ui/action-hover", "ui/action-pressed", "ui/focus",
                     "ui/hover", "ui/pressed", "ui/disabled", "ui/disabled-ink",
                     "ui/info", "ui/champagne"):
            self.assertIn(name, colors)
        self.assertEqual(sum(x["name"].startswith("Gold Heritage/")
                             for x in self.document["styles"]), 17)
        self.assertIn("GOLD HERITAGE  /  LIGHT", [
            x.get("characters") for x in self.nodes.values()
        ])

    def test_original_user_fixture_anonymized(self):
        visible = " | ".join(
            node.get("characters", "") for node in self.nodes.values()
            if node.get("type") == "text"
        )
        self.assertIn("Mira", visible)

    def test_six_source_based_circular_algorithm_lessons(self):
        lessons = [
            "91 Lesson - Anatomy of the wheel",
            "92 Lesson - Degrees to positions",
            "93 Lesson - Nakshatra and Pada",
            "94 Lesson - Planet markers",
            "95 Lesson - Houses and connectors",
            "96 Lesson - Assemble the wheel",
        ]
        self.assertEqual([p["name"] for p in self.pages[90:96]], lessons)
        for lesson in lessons:
            page_id = next(pid for pid in self.document["pages"]
                           if self.nodes[str(pid)]["name"] == lesson)
            frame = self.nodes[str(self.nodes[str(page_id)]["children"][0])]
            counts = Counter(self.nodes[str(child)]["name"] for child in frame["children"])
            self.assertEqual(counts["Pada / 108"], 108)
            self.assertEqual(counts["Nakshatra / 27"], 27)
            self.assertEqual(counts["Sign / 12"], 12)
            self.assertEqual((frame["width"], frame["height"]), (1440, 960))
        joined = " | ".join(node.get("characters", "")
                            for node in self.nodes.values() if node.get("type") == "text")
        for text in ["108 quarters / Padas", "Grid cell = 58",
                     "First cusp: r = 150", "Second: 198"]:
            self.assertIn(text, joined)

    def test_swiss_ephemeris_moon_explanation(self):
        page = self.pages[96]
        self.assertEqual(page["name"], "97 Lesson - Swiss Ephemeris to Moon")
        self.assertEqual(page["canvas"], [1440, 960])
        texts = [node.get("characters", "") for node in self.nodes.values()
                 if node.get("type") == "text"]
        joined = " | ".join(texts)
        for expected in (
            "219.03172767°", "23.78476711°", "195.24696056°",
            "jd = 2451545.0",
            "swe.set_sid_mode(swe.SIDM_LAHIRI)",
            "flags = swe.FLG_MOSEPH | swe.FLG_SIDEREAL",
            "moon, used = swe.calc_ut(jd, swe.MOON, flags)",
            "Swiss Ephemeris: 195.247°"
        ):
            self.assertIn(expected, joined)

    def test_geographic_house_cusps_and_connectors(self):
        lessons = [
            "98 Lesson - Location affects houses",
            "99 Lesson - Swiss house cusps",
            "100 Lesson - Cusps to connectors",
        ]
        self.assertEqual([p["name"] for p in self.pages[97:100]], lessons)
        content = " | ".join(n.get("characters", "") for n in self.nodes.values()
                             if n.get("type") == "text")
        for expected in (
            "154.614329°", "159.320911°", "184.614329°",
            "169.614329°", "195.246961°",
            "JD UT 2451545.0"
        ):
            self.assertIn(expected, content)
        f = self.nodes[str(self.nodes[str(self.document["pages"][99])]["children"][0])]
        names = Counter(self.nodes[str(cid)]["name"] for cid in f["children"])
        self.assertEqual(names["108 Pada division"], 108)
        self.assertEqual(names["12 sign division"], 12)
        self.assertEqual(sum(k.startswith("House connector / ") for k in names),12)
        self.assertEqual(sum(k.startswith("House midpoint / ") for k in names),12)
        self.assertEqual(sum(k.startswith("House cusp / ") for k in names),12)

    def test_native_visibility_and_layout_quality(self):
        """Regression: every user-visible label must be legible and inside its canvas."""
        def bounds(node):
            x, y = node.get("transform", [0, 0, 0, 0, 0, 0])[4:6]
            return (x, y, x + node.get("width", 0), y + node.get("height", 0))

        recovered_geo_pages = {
            "03 Aspects", "12 Tricakka", "13 House aspects",
            "14 Nakshatra traversal", "15 LatLon", "16 Western aspects",
            "17 Natal transit", "18 Rahu wheel", "19 Rasi strengths",
            "20 Transit planets", "21 Strength timeline", "22 Ashtakavarga",
            "23 KP subdivisions", "24 Heliocentric", "25 Divisional charts",
            "26 Interpretations", "27 Electional timing", "28 Ephemeris calendar",
            "29 Eclipses", "30 Transit events", "31 Profile library",
            "32 Location editor", "33 Calculation options", "34 Time navigator",
            "35 Unknown birth time", "36 Unsaved changes", "37 Backup restore",
            "38 Themes", "39 Period drilldown", "40 Period outside range",
            "50 Rahu manual reference", "51 Filtered strengths", "54 Chart zoom",
        }
        self.assertEqual(len(recovered_geo_pages), 33)
        framed_geo = 0
        theme_labels = 0
        for page_id in self.document["pages"]:
            page = self.nodes[str(page_id)]
            frame = self.nodes[str(page["children"][0])]
            visible = [self.nodes[str(cid)] for cid in frame["children"]
                       if self.nodes[str(cid)].get("visible", True)]
            texts = [node for node in visible if node["type"] == "text"]
            for node in visible:
                x, y, right, bottom = bounds(node)
                self.assertGreaterEqual(x, -1, (page["name"], node["name"]))
                self.assertGreaterEqual(y, -1, (page["name"], node["name"]))
                self.assertLessEqual(right, frame["width"] + 1,
                                     (page["name"], node["name"]))
                self.assertLessEqual(bottom, frame["height"] + 1,
                                     (page["name"], node["name"]))
            for idx, node in enumerate(texts):
                x, y, right, bottom = bounds(node)
                if page["name"] in recovered_geo_pages and node.get("characters", "").startswith(
                    "Yangon · 16.84° N, 96.17° E"
                ) and x == 780:
                    framed_geo += 1
                    # Later editable Lucide icons are in the sidebar, so the
                    # label need not be the frame's last child. It must remain
                    # above any later *opaque and overlapping* layer.
                    node_index = frame["children"].index(node["id"])
                    for following_id in frame["children"][node_index + 1:]:
                        top = self.nodes[str(following_id)]
                        if not top.get("visible", True):
                            continue
                        if top["type"] not in ("rectangle", "ellipse", "frame", "component"):
                            continue
                        fills = top.get("fills", [])
                        if not fills or not fills[0].get("visible", True):
                            continue
                        if fills[0].get("opacity", 1) < 0.96 or top.get("opacity", 1) < 0.96:
                            continue
                        x2, y2, r2, b2 = bounds(top)
                        dx = min(right, r2) - max(x, x2)
                        dy = min(bottom, b2) - max(y, y2)
                        self.assertFalse(
                            dx > 5 and dy > 5 and dx * dy > 100,
                            (page["name"], node["name"], top["name"]),
                        )
                if page["name"] in {
                    "56 Pink theme", "57 Blue theme", "58 Monochrome theme"
                } and node["name"] == "Chart geometry and values referenced from the supplied demo.":
                    theme_labels += 1
                    # Later editable Lucide icons are in the sidebar, so the
                    # label need not be the frame's last child. It must remain
                    # above any later *opaque and overlapping* layer.
                    node_index = frame["children"].index(node["id"])
                    for following_id in frame["children"][node_index + 1:]:
                        top = self.nodes[str(following_id)]
                        if not top.get("visible", True):
                            continue
                        if top["type"] not in ("rectangle", "ellipse", "frame", "component"):
                            continue
                        fills = top.get("fills", [])
                        if not fills or not fills[0].get("visible", True):
                            continue
                        if fills[0].get("opacity", 1) < 0.96 or top.get("opacity", 1) < 0.96:
                            continue
                        x2, y2, r2, b2 = bounds(top)
                        dx = min(right, r2) - max(x, x2)
                        dy = min(bottom, b2) - max(y, y2)
                        self.assertFalse(
                            dx > 5 and dy > 5 and dx * dy > 100,
                            (page["name"], node["name"], top["name"]),
                        )
                    self.assertEqual((x, y), (260, 813))
                for nxt in texts[idx + 1:]:
                    x2, y2, r2, b2 = bounds(nxt)
                    dx = min(right, r2) - max(x, x2)
                    dy = min(bottom, b2) - max(y, y2)
                    self.assertFalse(dx > 5 and dy > 5 and dx * dy > 125,
                        (page["name"], node["name"], nxt["name"]))
        self.assertEqual(framed_geo, 32)
        self.assertEqual(theme_labels, 3)

    def test_all_features_included_and_direct_mode_11_entry(self):
        """No product entitlement gate; original extra source mode opens directly."""
        policy = json.loads((ROOT / "design/APP_ROUTES.json").read_text())["access_policy"]
        self.assertEqual(policy["edition"], "free-for-all")
        self.assertFalse(policy["membership_required"])
        self.assertFalse(policy["activation_required"])
        self.assertFalse(policy["device_id_required"])
        self.assertFalse(policy["purchase_required"])
        self.assertFalse(policy["subscription_required"])
        self.assertTrue(policy["all_recovered_modes_included"])
        self.assertTrue(policy["all_tools_direct_mode_11"])

        alltools = self.nodes[str(self.document["pages"][10])]
        frame = self.nodes[str(alltools["children"][0])]
        mode11_frame = self.nodes[str(self.document["pages"][63])]["children"][0]
        entry = [
            self.nodes[str(cid)] for cid in frame["children"]
            if self.nodes[str(cid)]["name"] == "Navigate / Rasi aspects (included)"
        ]
        self.assertEqual(len(entry), 1)
        self.assertEqual(entry[0]["interactions"][0]["actions"][0]["dest"],
                         mode11_frame)
        text = [self.nodes[str(cid)].get("characters", "") for cid in frame["children"]
                if self.nodes[str(cid)]["type"] == "text"]
        self.assertIn("Rasi aspects", text)
        self.assertIn("Original mode 11 · included for all", text)
        for pageid in self.document["pages"][58:90]:
            page = self.nodes[str(pageid)]
            contents = [
                self.nodes[str(cid)].get("characters", "")
                for cid in self.nodes[str(page["children"][0])]["children"]
                if self.nodes[str(cid)]["type"] == "text"
            ]
            if not page["name"].startswith(("86 ", "87 ", "88 ")):
                self.assertIn("ALL FEATURES INCLUDED", contents)
            self.assertNotIn("Membership", contents)
        current_pages = [self.nodes[str(pid)]["name"]
                         for pid in self.document["pages"][58:64]]
        self.assertEqual(current_pages, [
            "59 All features included", "60 Included feature overview",
            "61 No account required", "62 Offline feature access",
            "63 Rasi aspects ready", "64 Rasi aspects mode",
        ])

    def test_genuine_lucide_svg_components_and_navigation(self):
        """Lucide icons must be actual native vectors with working prototype links."""
        import json
        contracts = json.loads((
            ROOT / "design/planning/native-precision-v2/TOKENS.json"
        ).read_text())
        names = {value["icon"] for value in contracts["icons"]["mapping"].values()}
        self.assertEqual(len(names), 30)
        icons = {
            node["id"]: node for node in self.nodes.values()
            if node["type"] == "component"
            and node.get("name", "").startswith("Lucide v0.468.0 / ")
        }
        self.assertEqual(len(icons), 70)
        self.assertEqual(len({icon["name"] for icon in icons.values()}), 35)
        self.assertEqual(set(Counter(icon["name"] for icon in icons.values()).values()), {2})
        for icon in icons.values():
            # Original Lucide SVG geometry imported as native vector/group layers.
            stack = list(icon.get("children", []))
            descendants = []
            while stack:
                current = self.nodes[str(stack.pop())]
                descendants.append(current)
                stack.extend(current.get("children", []))
            self.assertTrue(any(
                c["type"] in ("vector", "ellipse", "line") for c in descendants
            ), icon["name"])
            self.assertFalse(any(c["type"] == "image" for c in descendants))
        frame_for_workspaces = [
            self.nodes[str(self.nodes[str(pid)]["children"][0])]["id"]
            for pid in (
                self.document["pages"][0], self.document["pages"][30],
                self.document["pages"][32], self.document["pages"][33],
                self.document["pages"][10],
            )
        ]
        for pid in self.document["pages"][:90]:
            page = self.nodes[str(pid)]
            frame = self.nodes[str(page["children"][0])]
            refs = [
                self.nodes[str(cid)] for cid in frame["children"]
                if self.nodes[str(cid)]["type"] == "instance"
                and self.nodes[str(cid)].get("component") in icons
            ]
            self.assertEqual(len(refs), 5, page["name"])
            self.assertEqual(
                {ref["interactions"][0]["actions"][0]["dest"] for ref in refs},
                set(frame_for_workspaces),
                page["name"],
            )

    def test_native_precision_component_variants_and_semantic_tokens(self):
        """Reusable component assets and UI theme must not replace chart modes."""
        import json
        tokens = json.loads((
            ROOT / "design/planning/native-precision-v2/TOKENS.json"
        ).read_text())
        self.assertEqual(tokens["decisions"]["Q009"], "B")
        self.assertIsNone(tokens["color"]["dark"])
        families = [
            node for node in self.nodes.values()
            if node["type"] == "componentSet"
            and node.get("name", "").startswith("Native Precision / ")
        ]
        self.assertEqual(len(families), len(tokens["components"]) * 2)
        self.assertEqual(len({family["name"] for family in families}), len(tokens["components"]))
        self.assertEqual(set(Counter(family["name"] for family in families).values()), {2})
        expected_variants = {
            "State=Default", "State=Hover", "State=Focus", "State=Disabled"
        }
        for family in families:
            self.assertEqual(len(family["children"]), 4, family["name"])
            self.assertFalse(family.get("visible", True), family["name"])
            self.assertEqual({
                self.nodes[str(cid)]["name"] for cid in family["children"]
            }, expected_variants)
        css = (ROOT / "design/tokens.css").read_text()
        self.assertIn("--ui-action: #7C5B21;", css)
        self.assertIn("--ui-brand: #8B6B2E;", css)
        self.assertIn('data-motaa-chart-theme="pink"', css)
        self.assertIn('data-motaa-chart-theme="monochrome"', css)
        self.assertNotIn("data-motaa-appearance=\"dark\"", css)

    def test_macos_shell_across_all_product_states(self):
        """All 90 product/state references share macOS chrome; lessons remain separate."""
        self.assertEqual(len(self.document["pages"]), 111)
        names = [self.nodes[str(pid)]["name"] for pid in self.document["pages"]]
        self.assertEqual(len(names[:90]), 90)
        original_workspaces = [
            "01 Natal chart", "31 Profile library",
            "33 Calculation options", "34 Time navigator", "11 All tools",
        ]
        target_frames = [
            self.nodes[str(next(pid for pid in self.document["pages"]
                                if self.nodes[str(pid)]["name"] == name))]["children"][0]
            for name in original_workspaces
        ]
        for index, pid in enumerate(self.document["pages"][58:90], start=59):
            page = self.nodes[str(pid)]
            frame = self.nodes[str(page["children"][0])]
            children = [self.nodes[str(cid)] for cid in frame["children"]]
            lights = [node for node in children
                      if node.get("name", "").startswith("macOS window control / ")
                      and node.get("visible", True)]
            self.assertEqual(len(lights), 3, page["name"])
            selection = [node for node in children
                         if node.get("name", "").startswith("macOS selected workspace / ")]
            self.assertEqual(len(selection), 1, page["name"])
            nav = [node for node in children
                   if node.get("name", "").startswith("Navigate / ")
                   and node["type"] == "rectangle"
                   and node.get("transform", [0] * 6)[4] in (10, 18)
                   and 175 <= node.get("transform", [0] * 6)[5] <= 400
                   and node.get("visible", True)]
            self.assertEqual(len(nav), 5, page["name"])
            self.assertEqual(
                {node["interactions"][0]["actions"][0]["dest"] for node in nav},
                set(target_frames),
                page["name"],
            )
            badges = [node for node in children if node["type"] == "instance"
                      and node.get("name") in {
                          "MOTAA / Prototype status component",
                          "MOTAA / Source indicator component",
                      }]
            self.assertTrue(all(node.get("visible") is False for node in badges),
                            page["name"])
            visible_copy = " | ".join(node.get("characters", "") for node in children
                                      if node["type"] == "text"
                                      and node.get("visible", True))
            self.assertNotIn("DESKTOP WORKBOOK", visible_copy)
            self.assertNotIn("MOTAA / RECONSTRUCTION", visible_copy)

        birth_page = self.nodes[str(self.document["pages"][68])]
        birth_frame = self.nodes[str(birth_page["children"][0])]
        content = [self.nodes[str(cid)] for cid in birth_frame["children"]]
        labels = [node.get("name") for node in content
                  if node.get("name", "").startswith("Field label / ")]
        self.assertEqual(len(labels), 6)
        copy = [node.get("characters", "") for node in content
                if node["type"] == "text" and node.get("visible", True)]
        self.assertIn("Mandalay", copy)
        self.assertIn("21°58′ N", copy)
        self.assertIn("96°05′ E", copy)
        self.assertIn("21.9779, 96.0836", copy)

    def test_five_consolidated_native_application_flows(self):
        """The 111 FigCraft pages are five routes, 101 feature/state references, 10 lessons."""
        model = json.loads((ROOT / "design/APP_ROUTES.json").read_text())
        routes = model["routes"]
        self.assertEqual(len(routes), 5)
        all_states = [state for route in routes for state in route["states"]]
        self.assertEqual(len(all_states), 101)
        self.assertEqual(len({s["number"] for s in all_states}), 101)
        self.assertEqual(
            {state["name"] for state in all_states},
            {s["name"] for s in self.pages[:90] + self.pages[100:]},
        )
        self.assertEqual(model["educational_pages"],
                         [s["name"] for s in self.pages[90:100]])
        self.assertEqual(len(model["advertised_only_numbers"]), 16)
        self.assertEqual(len(model["recovered_modes"]), 11)
        self.assertEqual(len(model["conditional_reference_pages"]), 3)

        names = [p["name"] for p in self.pages]
        for route in routes:
            anchor = route["figcraft_page_name"]
            self.assertIn(anchor, names)
            page_id = next(
                pid for pid in self.document["pages"]
                if self.nodes[str(pid)]["name"] == anchor
            )
            page = self.nodes[str(page_id)]
            frames = [self.nodes[str(fid)] for fid in page["children"]]
            self.assertEqual(len(frames), 1)
            self.assertTrue(any(
                flow["name"] == route["figcraft_flow_name"]
                and flow["start"] == frames[0]["id"]
                and flow.get("description")
                for flow in page.get("flows", [])
            ), (route["slug"], page.get("flows")))
        all_app_flows = [
            flow
            for pid in self.document["pages"]
            for flow in self.nodes[str(pid)].get("flows", [])
            if flow["name"].startswith("APP / ")
        ]
        # Pages 106 and 108 retain duplicate native flow start points from their
        # source pages; they are additional entries, not additional workspaces.
        self.assertEqual(
            {flow["name"] for flow in all_app_flows},
            {route["figcraft_flow_name"] for route in routes},
        )
        gallery = (ROOT / "design/index.html").read_text()
        self.assertIn("5 application workspaces", gallery)
        self.assertIn("const routeGroups=", gallery)
        self.assertIn("const lessons=", gallery)
        self.assertIn("let selected=0;", gallery)

    def test_fully_source_derived_manifest_tokens_and_gallery(self):
        manifest = json.loads((ROOT / "design/source/screen-manifest.json").read_text())
        self.assertEqual(manifest, self.pages)
        self.assertEqual((ROOT / "design/tokens.css").read_text(), generate())
        gallery = (ROOT / "design/index.html").read_text()
        self.assertIn("111 screens", gallery)
        self.assertIn("item.canvas[0]", gallery)
        self.assertIn("item.canvas[1]", gallery)

    def test_display_text_size_and_reference_editing(self):
        variable = next(v for v in self.document["variables"]["variables"]
                        if v["name"] == "display/chart-text-size")
        choices = {}
        for node in self.nodes.values():
            for interaction in node.get("interactions", []):
                for action in interaction["actions"]:
                    if action.get("variable") == variable["id"]:
                        choices[node["characters"]] = action["value"]
        self.assertEqual(choices, {"Standard": 14, "Large": 17, "Extra large": 20,
                                   "Reset text size": 14})
        bound = [node for node in self.nodes.values()
                 if node.get("bound", {}).get("fontSize") == variable["id"]]
        self.assertEqual(len(bound), 50)
        self.assertTrue(all(node["height"] >= 28 for node in bound))

        pages = {page["name"]: page for page in self.pages}
        time_links = {link["label"]: link["destination"]
                      for link in pages["34 Time navigator"]["links"]}
        self.assertEqual(time_links["Edit reference location"], "70 Current coordinates")
        self.assertEqual(time_links["Edit reference date and time"], "72 Date and time editor")
        self.assertEqual(time_links["Settings"], "07 Settings")
        self.assertIn("34 Time navigator", {link["destination"]
                                            for link in pages["72 Date and time editor"]["links"]})

    def test_rasi_aspects_filter_switch_preserves_aspect_values(self):
        pages = {page["name"]: page for page in self.pages}
        filtered_name = "64 Rasi aspects mode"
        full_name = "111 Rasi aspects - all strengths"
        for source, target, control in ((filtered_name, full_name, "All strengths"),
                                        (full_name, filtered_name, "Aspected only")):
            links = {link["label"]: link["destination"] for link in pages[source]["links"]}
            self.assertEqual(links[f"Mode 11 / {control} button"], target)
            self.assertEqual(links["Mode 11 / Western aspects button"], "16 Western aspects")
            self.assertEqual(links["Mode 11 / All tools button"], "11 All tools")

        matrices = []
        for name in (filtered_name, full_name):
            page = next(self.nodes[str(pid)] for pid in self.document["pages"]
                        if self.nodes[str(pid)]["name"] == name)
            frame = self.nodes[str(page["children"][0])]
            table = {node["name"]: node.get("characters", "")
                     for node in (self.nodes[str(i)] for i in frame["children"])
                     if node["name"].startswith("Mode 11 / ") and " cell " in node["name"]}
            self.assertEqual(len(table), 9 * (10 + 12))
            matrices.append(table)
        filtered, full = matrices
        self.assertGreater(sum(bool(value) for value in full.values()),
                           sum(bool(value) for value in filtered.values()))
        self.assertTrue(all(full[cell] == value for cell, value in filtered.items() if value))

    def test_empty_library_centerline_and_compact_time_alignment(self):
        def center_x(node):
            return node["transform"][4] + node["width"] / 2

        content_center = center_x(self.nodes["2116"])
        for layer in (2120, 2129, 2130, 2131, 2132, 12780):
            self.assertAlmostEqual(center_x(self.nodes[str(layer)]), content_center)
        for button_value, hit_value in zip(self.nodes["2131"]["transform"],
                                           self.nodes["12780"]["transform"]):
            self.assertAlmostEqual(button_value, hit_value)
        for layer in (2129, 2130, 2132):
            self.assertEqual(self.nodes[str(layer)]["align_h"], "center")

        for page_id in self.document["pages"]:
            page = self.nodes[str(page_id)]
            frame = self.nodes[str(page["children"][0])]
            layers = [self.nodes[str(i)] for i in frame["children"]
                      if self.nodes[str(i)].get("visible", True)]
            for button, label in zip(layers, layers[1:]):
                if (label.get("characters") in ("‹", "›") and button["type"] == "rectangle"
                        and 28 <= button["width"] <= 52 and 28 <= button["height"] <= 46):
                    self.assertAlmostEqual(center_x(label), center_x(button), places=3)

        track = self.nodes["26699"]
        hit = self.nodes["26721"]
        step = self.nodes["26697"]
        date = self.nodes["26702"]
        self.assertEqual((track["transform"][4], track["width"]),
                         (hit["transform"][4], hit["width"]))
        self.assertGreaterEqual(track["transform"][4] - step["transform"][4] - step["width"], 16)
        self.assertGreaterEqual(date["transform"][4] - track["transform"][4] - track["width"], 16)

    def test_111_rendered_pngs_at_native_dimensions(self):
        for page in self.pages:
            path = ROOT / "design" / page["image"]
            with self.subTest(page=page["name"]):
                self.assertGreater(path.stat().st_size, 1000)
                with path.open("rb") as handle:
                    self.assertEqual(handle.read(8), b"\x89PNG\r\n\x1a\n")
                    handle.read(4)
                    self.assertEqual(handle.read(4), b"IHDR")
                    self.assertEqual(
                        list(struct.unpack(">II", handle.read(8))), page["canvas"]
                    )


    def test_native_option_rows_centered_without_flattening_fields(self):
        """Prevent the top-aligned, oversized option-label regression."""
        count = 0
        field_rows = 0
        pages = self.document["pages"]
        for pageid in pages:
            page = self.nodes[str(pageid)]
            num = int(page["name"].split()[0])
            if not 59 <= num <= 90:
                continue
            frame = self.nodes[str(page["children"][0])]
            children = [self.nodes[str(i)] for i in frame["children"]]
            texts = [v for v in children if v["type"] == "text"]
            for row in children:
                if row["type"] == "instance" and row["name"].startswith("Option /"):
                    main = self.nodes[str(row["component"])]
                    if main["name"].startswith("MOTAA / Option row /"):
                        label, = [self.nodes[str(i)] for i in row["children"]
                                  if self.nodes[str(i)]["type"] == "text"]
                        self.assertEqual(label["characters"], row.get("props", {}).get("Label", label["characters"]))
                        self.assertLessEqual(abs(label["transform"][5] - (row["height"] - label["height"]) / 2), 2.1)
                        self.assertGreaterEqual(label["style"]["font_size"], 14)
                        if num in range(59, 64):
                            self.assertLessEqual(row["height"], 52)
                        count += 1
                if (row["type"] != "rectangle" or not
                    row.get("name", "").startswith("Option /") or
                    row.get("width", 0) < 80):
                    continue
                x, y = row["transform"][4:6]
                w, h = row["width"], row["height"]
                inside = [v for v in texts
                          if x + 1 <= v["transform"][4] <= x + w - 1
                          and y - 1 <= v["transform"][5] <= y + h + 1]
                labels = [v for v in inside if v.get("name", "").startswith("Option label /")]
                if len(inside) == 1 and len(labels) == 1:
                    v = labels[0]
                    actual = v["transform"][5] - y
                    target = (h - v.get("height", 0)) / 2
                    self.assertLessEqual(abs(actual - target), 2.1,
                                         (page["name"], row["name"], actual, target))
                    self.assertGreaterEqual(v["style"]["font_size"], 14)
                    if num in range(59, 64):
                        self.assertLessEqual(h, 52)
                    count += 1
                elif num in (69, 70):
                    # Two-line label/value fields must keep their original hierarchy.
                    self.assertEqual(len(inside), 2, (page["name"], row["name"]))
                    field_rows += 1
        # Pages 87 and 88 now use four behavior cards plus a labeled window
        # preview instead of a fifth orphaned option card on each page.
        self.assertEqual(count, 136)
        self.assertEqual(field_rows, 12)

    def test_square_rasi_renderer_proportions_and_saturn_clearance(self):
        """main.java r0 uses offsets 340 and 610 across a 960-unit square."""
        original_nodes = []
        for page_id in self.document["pages"][:104]:
            stack = list(self.nodes[str(page_id)]["children"])
            while stack:
                node = self.nodes[str(stack.pop())]
                original_nodes.append(node)
                stack.extend(node.get("children", []))
        rasi = [node for node in original_nodes if node.get("characters") == "Rasi chart"
                and not node["name"].startswith("Mode 11 / ")]
        self.assertEqual(len(rasi), 18)
        expected = [
            (340, 0, 340, 960), (0, 340, 960, 340),
            (610, 0, 610, 960), (0, 610, 960, 610),
            (0, 0, 340, 340), (960, 0, 610, 340),
            (0, 960, 340, 610), (960, 960, 610, 610),
        ]
        for label in rasi:
            with self.subTest(label=label["id"]):
                parent = self.nodes[str(label["parent"])]
                parts = [self.nodes[str(node_id)] for node_id in parent["children"]]
                index = parts.index(label)
                border = parts[index - 10]
                self.assertEqual(border["type"], "rectangle")
                bx, by = border.get("transform", [1, 0, 0, 1, 0, 0])[4:]
                scale = border["width"] / 960
                for line, target in zip(parts[index - 9:index - 1], expected):
                    self.assertEqual(line["type"], "line")
                    cos, sin, _, _, x, y = line["transform"]
                    offsets = (x - bx, y - by, x - bx + cos * line["width"], y - by + sin * line["width"])
                    for actual, wanted in zip(offsets, target):
                        self.assertAlmostEqual(actual / scale, wanted, places=3)
                saturn = next(node for node in parts[index + 3:index + 23] if node.get("characters") == "0")
                value = parts[parts.index(saturn) + 1]
                self.assertEqual(value["characters"], "18°09′ R")
                for node in (saturn, value):
                    x, y = node["transform"][4:]
                    self.assertGreater((x - bx) / scale, 610)
                    right = (x + node["width"] - bx) / scale
                    bottom = (y + node["height"] - by) / scale
                    self.assertLess(bottom, (960 - right) * 340 / 350 - 8)

    def test_source_parity_correction_pass(self):
        variables = {item["name"]: item for item in self.document["variables"]["variables"]}
        self.assertNotIn("session/member", variables)
        self.assertIn("profiles.db + commments.db", " | ".join(
            node.get("characters", "") for node in self.nodes.values()
        ))
        self.assertIn("Desktop extension · not present in Android V31.1", [
            node.get("characters") for node in self.nodes.values()
        ])
        self.assertIn("534 source towns · offline directory · custom coordinates supported", [
            node.get("characters") for node in self.nodes.values()
        ])
        locations = json.loads((ROOT / "design/source/locations.v31.1.json").read_text())
        self.assertEqual(locations["town_count"], 534)
        self.assertEqual(len(locations["towns"]), 534)
        self.assertEqual(locations["towns"][0]["source_index"], 1)
        self.assertTrue(all(town["name_my"] for town in locations["towns"]))
        self.assertTrue((ROOT / "design/fonts/Astro.ttf").is_file())
        outlines = [node for node in self.nodes.values()
                    if node["name"].startswith("Astro outline / ")]
        self.assertEqual(len(outlines), 24)
        self.assertTrue(all(node["type"] == "vector" for node in outlines))
        self.assertEqual(Counter(node["parent"] for node in outlines),
                         {4462: 12, 13594: 12})
        self.assertFalse(any(node.get("style", {}).get("font_family") == "Astro"
                             for node in self.nodes.values()))
        self.assertEqual(self.nodes["805"]["characters"], "Swati")
        self.assertNotEqual(self.nodes["4165"]["characters"], "Pushya")
        original_ids = set()
        for page_id in self.document["pages"][:104]:
            stack = list(self.nodes[str(page_id)]["children"])
            while stack:
                node = self.nodes[str(stack.pop())]
                original_ids.add(node["id"])
                stack.extend(node.get("children", []))
        moon_signs = 0
        for node in self.nodes.values():
            if node["id"] not in original_ids or node.get("characters") != "15°14′":
                continue
            siblings = self.nodes[str(node["parent"])]["children"]
            previous = self.nodes[str(siblings[siblings.index(node["id"]) - 1])]
            if previous.get("name") == "2" or previous.get("characters") in {"2", "7"}:
                if previous.get("name") in {"2", "7"}:
                    self.assertEqual(previous["characters"], "7")
                    moon_signs += 1
        self.assertEqual(moon_signs, 18)

    def test_rahu_radii_match_source_scale(self):
        expected = (240, 220, 180, 125, 60)
        scale = 550 / 480
        for node_id, source_radius in zip((5182, 5183, 5184, 5185, 5186), expected):
            node = self.nodes[str(node_id)]
            self.assertAlmostEqual(node["width"] / 2, source_radius * scale, places=3)
            self.assertAlmostEqual(node["height"], node["width"], places=3)
            self.assertAlmostEqual(node["transform"][4] + node["width"] / 2, 505, places=3)
            self.assertAlmostEqual(node["transform"][5] + node["height"] / 2, 570, places=3)

    def test_independent_epoch_combinations_are_native_links(self):
        expected = {
            "N + N": ("Natal", "Natal", "74-natal-planets-natal-asc"),
            "N + T": ("Natal", "Transit", "75-natal-planets-transit-asc"),
            "T + N": ("Transit", "Natal", "76-transit-planets-natal-asc"),
            "T + T": ("Transit", "Transit", "77-transit-planets-transit-asc"),
        }
        frames = {node["name"]: node_id for node_id, node in self.nodes.items()
                  if node.get("type", "").lower() == "frame"}
        for label, (planets, ascendant, frame_name) in expected.items():
            links = [node for node in self.nodes.values()
                     if node.get("name") == f"Action / {label}" and node.get("interactions")]
            self.assertEqual(len(links), 4)
            for link in links:
                actions = link["interactions"][0]["actions"]
                self.assertEqual(actions[-1], {
                    "type": "navigate", "dest": int(frames[frame_name])
                })
                self.assertEqual(actions[0]["value"], planets)
                self.assertEqual(actions[1]["value"], ascendant)

    def test_workspace_toolbar_spacing(self):
        counts = Counter()
        for page_id in self.document["pages"]:
            page = self.nodes[str(page_id)]
            frame = self.nodes[str(page["children"][0])]
            children = list(screen_layers(self.nodes, frame))
            visible = {node["name"]: node for node in children if node.get("visible", True)}
            status = visible.get("Local workspace")
            if status is None:
                if page["name"] == "88 Desktop 1024":
                    rail_width = visible["Navigation rail"]["width"]
                    self.assertEqual(visible["Breadcrumb"]["transform"][4], rail_width + 30)
                    controls = [visible[f"macOS window control / {i}"] for i in range(3)]
                    for control in controls:
                        x = control["transform"][4]
                        self.assertGreaterEqual(x, 8)
                        self.assertLessEqual(x + control["width"], rail_width - 8)
                continue
            with self.subTest(page=page["name"]):
                sx, sy = status["transform"][4:]
                self.assertEqual(frame["width"] - sx - status["width"], 34)
                self.assertEqual(sy + status["height"] / 2, 30)
                dots = [node for node in children if node["type"] == "ellipse" and node.get("visible", True)
                        and node.get("transform", [])[4:6] == [sx - 14, 27]]
                self.assertEqual(len(dots), 1)
                self.assertAlmostEqual(dots[0]["width"], 6)
                counts["status"] += 1
                button = visible.get("Birth details visibility button")
                if button:
                    bx, by = button["transform"][4:]
                    self.assertEqual((button["width"], button["height"]), (144, 34))
                    self.assertEqual(by + button["height"] / 2, 30)
                    self.assertEqual(sx - 14 - bx - button["width"], 18)
                    label = visible["Birth details visibility label"]
                    self.assertEqual(label["transform"][4:], [bx + 4, by])
                    self.assertEqual((label["width"], label["height"]), (136, 34))
                    self.assertEqual((label["align_h"], label["align_v"]), ("center", "middle"))
                    counts["privacy"] += 1
        self.assertEqual(counts, {"status": 67, "privacy": 55})

    def test_native_action_labels_centered_inside_buttons(self):
        counts = Counter()
        for page_id in self.document["pages"][58:90]:
            page = self.nodes[str(page_id)]
            frame = self.nodes[str(page["children"][0])]
            children = list(screen_layers(self.nodes, frame))
            labels = {node["name"]: node for node in children
                      if node["type"] == "text" and node.get("visible", True)}
            for button in children:
                if (button["type"] != "rectangle" or not button.get("visible", True)
                        or not button.get("interactions") or not button["name"].startswith("Action / ")):
                    continue
                label = labels.get("Label / " + button["name"].removeprefix("Action / "))
                if label is None or not label.get("characters"):
                    continue
                with self.subTest(page=page["name"], label=label["characters"]):
                    x, y = button["transform"][4:]
                    self.assertEqual(label["transform"][4:], [x + 4, y])
                    self.assertEqual(label["width"], button["width"] - 8)
                    self.assertEqual(label["height"], button["height"])
                    self.assertEqual(label["align_h"], "center")
                    self.assertEqual(label["align_v"], "middle")
                    self.assertEqual(label["auto_resize"], "none")
                counts[page_id] += 1
        self.assertEqual(sum(counts.values()), 129)
        self.assertEqual(len(counts), 32)

    def test_new_source_parity_states_and_prototype_reachability(self):
        expected = {"111 Rasi aspects - all strengths", "101 Myanmar lunar calendar", "102 Original calculation controls",
                    "103 Traditional observances", "104 Sky map - 2D",
                    "105 First launch - anonymous sample", "106 Profile contexts - Natal and Transit",
                    "107 Sky map - 3D", "108 Compact chart - 1280x800",
                    "109 Advanced calculation - recoverable error",
                    "110 Design system - component states"}
        self.assertEqual({x["name"] for x in self.pages[100:]}, expected)
        text = " | ".join(x.get("characters", "") for x in self.nodes.values()
                          if x.get("type") == "text")
        for label in ("Myanmar calendar", "Gregorian", "Julian", "Hybrid",
                      "5D_Aspect", "15D_Aspect", "Yatyaza", "Pyathada",
                      "Sabbath Eve", "Thamanyo", "Sky map", "3D sky view"):
            self.assertIn(label, text)
        frames = {}
        for pid in self.document["pages"]:
            page = self.nodes[str(pid)]
            frames[page["children"][0]] = int(page["name"].split()[0])
        by_page = {}
        for pid in self.document["pages"]:
            page = self.nodes[str(pid)]
            current = int(page["name"].split()[0])
            stack, edges = list(page["children"]), set()
            while stack:
                node = self.nodes[str(stack.pop())]
                stack.extend(node.get("children", []))
                for interaction in node.get("interactions", []):
                    for action in interaction.get("actions", []):
                        if action["type"] == "navigate":
                            self.assertIn(action["dest"], frames)
                            edges.add(frames[action["dest"]])
            by_page[current] = edges
        reached, pending = {1}, [1]
        while pending:
            for destination in by_page[pending.pop()]:
                if destination not in reached:
                    reached.add(destination)
                    pending.append(destination)
        self.assertEqual(reached, set(range(1, 112)))


    def test_primitive_semantic_component_token_aliases(self):
        """Raw primitives feed preserved semantic IDs and component contracts."""
        data = self.document["variables"]
        variables = {item["name"]: item for item in data["variables"]}
        by_id = {item["id"]: item for item in data["variables"]}
        collections = {item["id"]: item for item in data["collections"]}
        self.assertEqual(variables["ui/window"]["id"], "VariableID:28")
        for name in ("ui/window", "ui/action", "component/action/primary/background",
                     "component/control/height", "component/motion/duration"):
            self.assertIn(name, variables)
        def resolve(name):
            variable = variables[name]
            mode_id = collections[variable["collection"]]["modes"][0]["id"]
            seen = set()
            while True:
                self.assertNotIn(variable["id"], seen)
                seen.add(variable["id"])
                value = variable["values"][mode_id]
                if value["type"] != "alias":
                    return value
                variable = by_id[value["value"]]
                modes = collections[variable["collection"]]["modes"]
                if mode_id not in variable["values"]:
                    self.assertEqual(len(modes), 1)
                    mode_id = modes[0]["id"]
        window = resolve("ui/window")["value"]
        self.assertEqual("#" + "".join(f"{round(window[ch] * 255):02X}" for ch in "rgb"), "#F8F5EE")
        action = resolve("component/action/primary/background")["value"]
        self.assertEqual("#" + "".join(f"{round(action[ch] * 255):02X}" for ch in "rgb"), "#7C5B21")
        self.assertEqual(resolve("component/control/height")["value"], 38)
        self.assertEqual(resolve("component/motion/duration")["value"], 180)
        self.assertEqual(resolve("ui/window")["type"], "color")

    def test_free_enabled_ux_workflows_and_recovery_links(self):
        expected = [
            "105 First launch - anonymous sample", "106 Profile contexts - Natal and Transit",
            "107 Sky map - 3D", "108 Compact chart - 1280x800",
            "109 Advanced calculation - recoverable error",
            "110 Design system - component states",
        ]
        self.assertEqual([page["name"] for page in self.pages[104:110]], expected)
        visible_text = " | ".join(
            node.get("characters", "") for node in self.nodes.values()
            if node.get("type") == "text" and node.get("visible", True)
        )
        for label in ("Explore a sample chart", "Assign Natal and Transit profiles",
                      "3D celestial view · included free", "The calculation couldn’t finish",
                      "Keyboard focus", "Calculating…", "Every error names the preservation guarantee"):
            self.assertIn(label, visible_text)
        self.assertNotIn("SIBLING · RESEARCH", visible_text)
        frames = {}
        for pid in self.document["pages"]:
            page = self.nodes[str(pid)]
            frames[page["children"][0]] = page["name"]
        def destinations(page_name, node_name):
            page_id = next(pid for pid in self.document["pages"]
                           if self.nodes[str(pid)]["name"] == page_name)
            frame_id = self.nodes[str(page_id)]["children"][0]
            stack, result = [frame_id], set()
            while stack:
                node = self.nodes[str(stack.pop())]
                stack.extend(node.get("children", []))
                if node.get("name") == node_name:
                    for interaction in node.get("interactions", []):
                        for action in interaction.get("actions", []):
                            if action["type"] == "navigate":
                                result.add(frames[action["dest"]])
            return result
        checks = [
            ("31 Profile library", "UX / Manage contexts button", "106 Profile contexts - Natal and Transit"),
            ("104 Sky map - 2D", "NAV | Chart modes", "107 Sky map - 3D"),
            ("107 Sky map - 3D", "NAV | Chart modes", "104 Sky map - 2D"),
            ("01 Natal chart", "UX / Open compact chart button", "108 Compact chart - 1280x800"),
            ("108 Compact chart - 1280x800", "UX / Open full chart button", "01 Natal chart"),
            ("08 Empty library", "UX / Preview first launch button", "105 First launch - anonymous sample"),
            ("105 First launch - anonymous sample", "UX / First launch create button", "06 Birth profile"),
            ("105 First launch - anonymous sample", "UX / First launch import button", "37 Backup restore"),
            ("28 Ephemeris calendar", "UX / Preview calculation recovery button", "109 Advanced calculation - recoverable error"),
            ("109 Advanced calculation - recoverable error", "UX / Open recovery calculation options button", "33 Calculation options"),
            ("86 Design foundations", "UX / Open component states button", "110 Design system - component states"),
        ]
        for page, node, destination in checks:
            with self.subTest(page=page, node=node):
                self.assertEqual(destinations(page, node), {destination})
        policy = json.loads((ROOT / "design/APP_ROUTES.json").read_text())["access_policy"]["advanced_research_policy"]
        self.assertTrue(policy["free_access"])
        self.assertFalse(policy["hidden"])
        self.assertFalse(policy["fake_calculations"])
        self.assertTrue(policy["enabled_2d_3d_sky_workflows"])

    def test_compact_native_chart_and_visible_component_states(self):
        compact = next(page for page in self.pages if page["name"] == "108 Compact chart - 1280x800")
        self.assertEqual(compact["canvas"], [1280, 800])
        page_id = next(pid for pid in self.document["pages"]
                       if self.nodes[str(pid)]["name"] == compact["name"])
        frame = self.nodes[str(self.nodes[str(page_id)]["children"][0])]
        self.assertEqual((frame["width"], frame["height"]), (1280, 800))
        chart = self.nodes[str(next(cid for cid in frame["children"]
                                    if self.nodes[str(cid)].get("type") == "rectangle"
                                    and abs(self.nodes[str(cid)].get("width", 0) - 518 * 0.80) < 0.01))]
        self.assertAlmostEqual(chart["width"], 518 * 0.80)
        self.assertAlmostEqual(chart["height"], 518 * 0.80)
        self.assertAlmostEqual(chart["transform"][5], 270)
        self.assertFalse(any(self.nodes[str(cid)].get("type") == "image" for cid in frame["children"]))
        state_text = " | ".join(
            node.get("characters", "") for node in self.nodes.values()
            if node.get("type") == "text" and node.get("name", "").startswith("UX / State")
        )
        for label in ("Default", "Hover", "Pressed", "Keyboard focus", "Disabled", "Calculating…"):
            self.assertIn(label, state_text)

    def test_nested_effective_visibility_and_opaque_cover(self):
        """Catch hidden/nested layout defects that top-level-only checks miss."""
        def multiply(parent, child):
            a, b, c, d, e, f = parent
            g, h, i, j, k, l = child
            return [
                a * g + c * h, b * g + d * h,
                a * i + c * j, b * i + d * j,
                a * k + c * l + e, b * k + d * l + f,
            ]

        def bounds(node, matrix):
            points = []
            for x, y in ((0, 0), (node.get("width", 0), 0),
                         (0, node.get("height", 0)),
                         (node.get("width", 0), node.get("height", 0))):
                a, b, c, d, e, f = matrix
                points.append((a * x + c * y + e, b * x + d * y + f))
            xs, ys = [point[0] for point in points], [point[1] for point in points]
            return [min(xs), min(ys), max(xs), max(ys)]

        def overlap(first, second):
            dx = min(first[2], second[2]) - max(first[0], second[0])
            dy = min(first[3], second[3]) - max(first[1], second[1])
            return dx, dy, dx * dy

        def opaque(node):
            if node.get("opacity", 1) < 0.96:
                return False
            return any(
                fill.get("visible", True) and fill.get("opacity", 1) >= 0.96
                and fill.get("color", {}).get("a", 1) >= 0.96
                for fill in node.get("fills", [])
            )

        for page_id in self.document["pages"]:
            page = self.nodes[str(page_id)]
            frame = self.nodes[str(page["children"][0])]
            flat = []

            def walk(node_id, matrix, visible, ancestors):
                node = self.nodes[str(node_id)]
                matrix = multiply(matrix, node.get("transform", [1, 0, 0, 1, 0, 0]))
                visible = visible and node.get("visible", True)
                if visible:
                    flat.append({
                        "node": node, "bounds": bounds(node, matrix),
                        "ancestors": ancestors,
                    })
                for child_id in node.get("children", []):
                    walk(child_id, matrix, visible, ancestors + [node["id"]])

            for child_id in frame.get("children", []):
                walk(child_id, [1, 0, 0, 1, 0, 0], True, [frame["id"]])

            texts = [item for item in flat if item["node"]["type"] == "text"]
            for item in flat:
                node = item["node"]
                x, y, right, bottom = item["bounds"]
                self.assertGreaterEqual(x, -1, (page["name"], node["name"], "nested x"))
                self.assertGreaterEqual(y, -1, (page["name"], node["name"], "nested y"))
                self.assertLessEqual(right, frame["width"] + 1,
                                     (page["name"], node["name"], "nested right"))
                self.assertLessEqual(bottom, frame["height"] + 1,
                                     (page["name"], node["name"], "nested bottom"))

            for index, first in enumerate(texts):
                for second in texts[index + 1:]:
                    dx, dy, area = overlap(first["bounds"], second["bounds"])
                    self.assertFalse(dx > 5 and dy > 5 and area > 125, (
                        page["name"], first["node"]["name"], second["node"]["name"],
                        "nested text overlap",
                    ))

            for index, text in enumerate(flat):
                if text["node"]["type"] != "text":
                    continue
                for top in flat[index + 1:]:
                    node = top["node"]
                    if node["type"] not in ("rectangle", "frame", "ellipse", "component"):
                        continue
                    if not opaque(node) or text["node"]["id"] in top["ancestors"]:
                        continue
                    dx, dy, area = overlap(text["bounds"], top["bounds"])
                    if node["type"] == "ellipse":
                        # Bounding boxes at a circle's tangent are not covered ink.
                        tx = (text["bounds"][0] + text["bounds"][2]) / 2
                        ty = (text["bounds"][1] + text["bounds"][3]) / 2
                        x, y, right, bottom = top["bounds"]
                        rx, ry = (right - x) / 2, (bottom - y) / 2
                        if rx <= 0 or ry <= 0:
                            continue
                        inside = ((tx - (x + rx)) / rx) ** 2 + ((ty - (y + ry)) / ry) ** 2
                        if inside > 1:
                            continue
                    self.assertFalse(dx > 5 and dy > 5 and area > 125, (
                        page["name"], text["node"]["name"], node["name"],
                        "opaque layer covers text",
                    ))

    def test_layout_collision_regressions(self):
        """Pinned repairs for the 2026-11-21 full-page visual review."""
        for rectangle, label in ((17609, 17610), (17611, 17612), (17613, 17614)):
            rect = self.nodes[str(rectangle)]
            text = self.nodes[str(label)]
            self.assertLessEqual(rect["transform"][5] + rect["height"],
                                 self.nodes[str(232 if rectangle == 17609 else
                                                 2325 if rectangle == 17611 else 15667)]
                                 ["transform"][5])
            self.assertEqual(text["transform"][5], rect["transform"][5] + 10)

        moon_badge = self.nodes["9873"]
        burmese_letter = self.nodes["9848"]
        self.assertLessEqual(moon_badge["transform"][4] + moon_badge["width"],
                             burmese_letter["transform"][4])

        equals = self.nodes["17647"]
        result_card = self.nodes["17648"]
        self.assertLessEqual(equals["transform"][4] + equals["width"],
                             result_card["transform"][4])

        divider = self.nodes["26532"]
        self.assertEqual((divider["width"], divider["height"]), (800, 0))
        chart = self.nodes["26610"]
        for control_id in (26599, 26604):
            control = self.nodes[str(control_id)]
            self.assertLessEqual(control["transform"][5] + control["height"],
                                 chart["transform"][5])
        footer = self.nodes["26689"]
        for caption_id in (26684, 26685, 26686):
            caption = self.nodes[str(caption_id)]
            self.assertLessEqual(caption["transform"][5] + caption["height"],
                                 footer["transform"][5])
        self.assertLessEqual(chart["transform"][5] + chart["height"],
                             footer["transform"][5])



    def test_post_delete_state_and_related_screen_annotations(self):
        """Deletion is a completed state with coherent copy and one action group."""
        subtitle = self.nodes["10548"]
        self.assertIn("was removed from this device", subtitle["characters"])
        self.assertNotIn("Review the record", subtitle["characters"])
        self.assertIn("Recently deleted", self.nodes["10555"]["characters"])

        card = self.nodes["10559"]
        card_bottom = card["transform"][5] + card["height"]
        undo = self.nodes["10562"]
        return_to_profiles = self.nodes["10564"]
        self.assertEqual(undo["transform"][5], return_to_profiles["transform"][5])
        self.assertEqual(undo["height"], return_to_profiles["height"])
        self.assertLessEqual(undo["transform"][4] + undo["width"],
                             return_to_profiles["transform"][4])
        for visual, hotspot in ((10562, 13392), (10564, 13393)):
            visual_node = self.nodes[str(visual)]
            hotspot_node = self.nodes[str(hotspot)]
            self.assertEqual(visual_node["transform"], hotspot_node["transform"])
            self.assertAlmostEqual(visual_node["width"], hotspot_node["width"])
            self.assertAlmostEqual(visual_node["height"], hotspot_node["height"])
            self.assertGreaterEqual(visual_node["transform"][4]
                                    - card["transform"][4], 24)
            self.assertGreaterEqual(card_bottom - (visual_node["transform"][5]
                                                   + visual_node["height"]), 24)

        self.assertIn("Only this local profile is removed",
                      self.nodes["10482"]["characters"])
        for orphan_label in (10487, 10566, 11135, 11206):
            self.assertFalse(self.nodes[str(orphan_label)].get("visible", True))

    def test_breakpoint_static_reference_composition(self):
        """Breakpoint pages are balanced, labeled static design specifications."""
        cases = (
            {
                "subtitle": 15541,
                "heading": 15547,
                "heading_text": "1280 × 960 · Medium desktop",
                "preview": 16068,
                "preview_box": (245, 340, 707, 174),
                "navigation": 15577,
                "cards": (15565, 15568, 15571, 15574),
                "markers": (15553, 15555, 15557, 25285, 25287, 24790, 24795, 24792),
                "status": 15558,
            },
            {
                "subtitle": 15608,
                "heading": 15614,
                "heading_text": "1024 × 900 · Compact desktop",
                "preview": 16080,
                "preview_box": (121, 340, 613, 174),
                "navigation": 15644,
                "cards": (15632, 15635, 15638, 15641),
                "markers": (15620, 15622, 15624, 24797, 24799, 24801, 24803, 24805),
                "status": 15625,
            },
        )
        for case in cases:
            with self.subTest(preview=case["preview"]):
                self.assertIn("specified, not simulated",
                              self.nodes[str(case["subtitle"])]["characters"])
                self.assertEqual(self.nodes[str(case["heading"])]["characters"],
                                 case["heading_text"])
                preview = self.nodes[str(case["preview"])]
                self.assertEqual(
                    (preview["transform"][4], preview["transform"][5],
                     preview["width"], preview["height"]),
                    case["preview_box"],
                )
                navigation = self.nodes[str(case["navigation"])]
                self.assertTrue(navigation["name"].startswith("Preview /"))
                self.assertEqual(navigation["transform"][5], preview["transform"][5])
                self.assertEqual(navigation["height"], preview["height"])

                cards = [self.nodes[str(node_id)] for node_id in case["cards"]]
                self.assertEqual(cards[0]["transform"][5], cards[1]["transform"][5])
                self.assertEqual(cards[2]["transform"][5], cards[3]["transform"][5])
                self.assertEqual(cards[0]["transform"][4], cards[2]["transform"][4])
                self.assertEqual(cards[1]["transform"][4], cards[3]["transform"][4])
                self.assertLess(cards[0]["transform"][5], cards[2]["transform"][5])
                for card in cards:
                    self.assertTrue(card.get("visible", True))
                    self.assertEqual(card["bound"]["fill"], "VariableID:30")
                for marker in case["markers"]:
                    self.assertFalse(self.nodes[str(marker)].get("visible", True))
                self.assertIn("Static", self.nodes[str(case["status"])]["characters"])
                self.assertIn("No responsive runtime claim",
                              self.nodes[str(case["status"])]["characters"])

    def test_component_states_share_one_grid_and_complete_controls(self):
        """Component specimens use consistent columns and complete visible states."""
        panels = [self.nodes[str(node_id)] for node_id in (27970, 27987, 28001, 28016)]
        self.assertEqual(
            [(panel["transform"][4], panel["transform"][5],
              panel["width"], panel["height"]) for panel in panels],
            [(240, 214, 571, 292), (835, 214, 571, 292),
             (240, 538, 571, 316), (835, 538, 571, 316)],
        )

        buttons = [self.nodes[str(node_id)] for node_id in (27972, 27974, 27976)]
        self.assertEqual([button["transform"][4] for button in buttons],
                         [264, 442.333, 620.667])
        for button in buttons:
            self.assertAlmostEqual(button["width"], 166.333)
            self.assertEqual(button["height"], 38)

        self.assertEqual(self.nodes["27979"]["width"], buttons[0]["width"])
        self.assertEqual(self.nodes["27979"]["transform"][4], buttons[0]["transform"][4])
        for title in (28005, 28009, 28013):
            self.assertEqual(self.nodes[str(title)]["align_v"], "middle")

        error_input = self.nodes["27997"]
        success_input = self.nodes["27156"]
        success_label = self.nodes["27161"]
        success_value = self.nodes["27158"]
        success_help = self.nodes["28000"]
        for node in (success_input, success_label, success_value, success_help):
            self.assertTrue(node.get("visible", True))
        self.assertEqual(success_label["characters"], "Success")
        self.assertEqual(success_value["characters"], "20.00000")
        self.assertEqual(success_value["align_v"], "middle")
        self.assertEqual(success_input["transform"][5], error_input["transform"][5])
        self.assertEqual(success_input["width"], error_input["width"])
        self.assertEqual(success_input["height"], error_input["height"])
        self.assertEqual(success_help["transform"][5], self.nodes["27999"]["transform"][5])
        self.assertEqual(success_help["transform"][4], success_input["transform"][4])

        feedback_rows = [self.nodes[str(node_id)] for node_id in (28003, 28007, 28011)]
        feedback_copy = [self.nodes[str(node_id)] for node_id in (28006, 28010, 28014)]
        for row in feedback_rows:
            self.assertEqual((row["transform"][4], row["width"]), (264, 523))
        for copy in feedback_copy:
            self.assertEqual((copy["transform"][4], copy["width"]), (394, 369))

        selection_rows = [self.nodes[str(node_id)] for node_id in (28018, 28020, 28022, 28024)]
        for index, row in enumerate(selection_rows):
            self.assertEqual((row["transform"][4], row["transform"][5],
                              row["width"], row["height"]),
                             (859, 602 + index * 48, 523, 38))
            self.assertTrue(row.get("strokes"))
            self.assertEqual(row["corners"]["radii"], [6, 6, 6, 6])
            self.assertNotEqual(row["fills"], panels[0]["fills"])


if __name__ == "__main__":
    unittest.main()
