#!/usr/bin/env python3
"""Capture synthetic calculations from the exact MOTAA V31.1 Android artifact."""

import argparse
import json
from pathlib import Path
import re
import subprocess
import tempfile
import uuid

SOURCE = Path(__file__).resolve().parents[1] / "tests/oracles/MotaaNumericOracle.java"
SHA256 = "f68d54931c721d82c8056a919e679c26c36113c3b191a575f32dd227b8f31734"
MODES = (
    "LAHIRI", "FAGANBRADLEY", "DELUCE", "RAMAN", "USHASHASHI",
    "KRISHNAMURTI", "DJWHALKHUL", "YUKTESHWAR", "JNBHASIN",
)
SYSTEMS = (
    "Equal", "Vehlow_Placidus", "Placidus", "Koch", "Porphyrius",
    "Regiomontanus", "Campanus", "Vehlow_Equal",
)
# Civil date and time, minutes east of UTC, coordinates, tropical flag and mode.
CASES = (
    (1984, 1, 3, 5, 45, 0, 390, 21.535556, 95.201667, False, 0),
    (2000, 1, 1, 12, 0, 0, 0, 16.84, 96.17, True, 0),
    (2024, 4, 8, 18, 17, 18, 0, 25.287, -104.138, False, 1),
    (1980, 6, 21, 18, 0, 0, -180, -34.6, -58.4, False, 2),
    (1900, 1, 15, 0, 0, 0, 0, 51.5, -0.12, False, 3),
    (2090, 10, 10, 18, 0, 0, 330, 12.97, 77.59, False, 4),
    (2013, 5, 8, 3, 18, 7, 420, 13.75, 100.50, False, 5),
    (1974, 12, 2, 6, 20, 19, 60, 48.86, 2.35, False, 6),
    (2034, 3, 7, 20, 12, 0, -240, 40.71, -74.01, False, 7),
    (1885, 7, 14, 15, 49, 12, 0, -33.87, 151.21, False, 8),
)
EDGE_CASES = (
    (1885, 1, 1, 0, 0, 0, 0, 64.1466, -21.9426, True, 0),
    (2099, 12, 31, 23, 59, 59, 840, -41.2866, 174.7756, False, 0),
    (2024, 6, 20, 20, 51, 0, 120, 69.6492, 18.9553, False, 0),
    (2024, 1, 15, 12, 0, 0, 345, 27.7172, 85.3240, False, 0),
)



def capture(adb, apk, remote, cases):
    records = []
    for y, m, d, h, minute, second, offset, lat, lon, tropical, mode in cases:
        args = (
            y, m, d, h + minute / 60 + second / 3600,
            offset / 60, lat, lon, "Tropical" if tropical else "Sidereal", MODES[mode],
        )
        output = subprocess.check_output(
            [*adb, "shell", "dalvikvm", "-cp", f"{remote}:{apk}", "MotaaNumericOracle",
             *map(str, args)], text=True, timeout=30,
        )
        data = {}
        for line in output.splitlines():
            if item := re.fullmatch(r"([A-Za-z_]+)=(\[.*\])", line):
                data[item[1]] = json.loads(item[2])
        if len(data.get("positions", [])) != 12 or len(data.get("angles", [])) != 10:
            raise ValueError("Android oracle returned incomplete positions")
        if any(len(data.get(system, [])) != 13 for system in SYSTEMS):
            raise ValueError("Android oracle returned incomplete house cusps")
        records.append(dict(
            year=y, month=m, day=d, hour=h, minute=minute, second=second,
            offset=offset, latitude=lat, longitude=lon, tropical=tropical,
            mode=mode, positions=data["positions"], tropical_positions=data["tropical_positions"],
            angle=data["angles"][0],
            houses=[data[system][1:] for system in SYSTEMS], western=data["western"],
            moon_clock=data["moon_clock"], moon_state=data["moon_state"], dasha=data["dasha"], speeds=data["speeds"], declinations=data["declinations"], distances=data["distances"],
            house_midpoints=[data[f"centers_{system}"] for system in SYSTEMS],
            western_all_houses=[data[f"western_{system}"] for system in SYSTEMS],
        ))
    return records


def capture_profile_database(adb, remote, destination):
    # Table definitions from MOTAA V31.1 L0.p.onCreate. Values are synthetic.
    profile = (
        "Birth Name", "3", "1", "1984", "5", "45", "0", "6", "30", "EGMT",
        "Birth Place", "21", "32", "8", "N", "95", "11", "56", "E", "Sidereal",
        "LAHIRI", "Vehlow_Equal", "True_Rahu", "5D_Aspect",
    )
    commands = [
        ".bail on",
        "BEGIN;",
        "create table MY_TABLE (Content text not null);",
        "create table comments( _id integer primary key autoincrement, comment text not null);",
        *[f"INSERT INTO MY_TABLE VALUES ('{field}');" for field in profile],
        "INSERT INTO comments(comment) VALUES ('Natal Record 15 6 1990 12 20 4 5 0 WGMT Boston 42 0 0 N 71 0 0 W Tropical RAMAN Porphyrius Mean_Rahu 15D_Aspect 2 Current Place 6 30 EGMT 16 48 0 N 96 10 0 E');",
        "COMMIT;",
    ]
    subprocess.run([*adb, "shell", "-T", "sqlite3", remote],
                   input="\n".join(commands) + "\n", text=True, check=True, timeout=30)
    subprocess.run([*adb, "pull", remote, destination], check=True, timeout=30)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--serial", required=True)
    parser.add_argument("--d8", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--include-edge-cases", action="store_true")
    parser.add_argument("--profile-database", type=Path)
    parser.add_argument("--calendar", type=Path)
    parser.add_argument("--myanmar", type=Path)
    parser.add_argument("--rahu", type=Path)
    parser.add_argument("--delta-t", type=Path)
    parser.add_argument("--ayanamsha", type=Path)
    args = parser.parse_args()
    for destination in (args.output, args.profile_database, args.calendar, args.myanmar, args.rahu, args.delta_t, args.ayanamsha):
        if destination is not None and destination.exists():
            raise FileExistsError(destination)
    if not re.fullmatch(r"emulator-[0-9]+", args.serial):
        raise ValueError("Select an Android emulator serial")
    adb = ["adb", "-s", args.serial]
    packages = subprocess.check_output(
        [*adb, "shell", "pm", "path", "motaa.myanmar"], text=True, timeout=15
    )
    apk = next(
        entry.removeprefix("package:") for entry in packages.splitlines()
        if entry.startswith("package:") and entry.endswith("/base.apk")
    )
    digest = subprocess.check_output(
        [*adb, "shell", "sha256sum", apk], text=True, timeout=30
    ).split()[0]
    if digest != SHA256:
        raise ValueError(f"Unexpected APK hash {digest}")
    remote = f"/data/local/tmp/motaa-oracle-{uuid.uuid4().hex}.zip"
    remote_db = remote + ".sqlite"
    with tempfile.TemporaryDirectory(prefix="motaa-oracle-") as directory:
        work = Path(directory)
        archive = work / "oracle.zip"
        subprocess.run(["javac", "--release", "8", "-Xlint:-options", "-d", work, SOURCE],
                       check=True, timeout=30)
        subprocess.run([args.d8, "--output", archive, work / "MotaaNumericOracle.class"],
                       check=True, timeout=30)
        try:
            subprocess.run([*adb, "push", archive, remote], check=True, timeout=30)
            records = capture(adb, apk, remote, CASES + EDGE_CASES if args.include_edge_cases else CASES)
            if args.calendar:
                raw_calendar = subprocess.check_output(
                    [*adb, "shell", "dalvikvm", "-cp", f"{remote}:{apk}",
                     "MotaaNumericOracle", "calendar"], text=True, timeout=30,
                )
                entries = [json.loads(line.removeprefix("calendar="))
                           for line in raw_calendar.splitlines() if line.startswith("calendar=")]
                if len(entries) != 15:
                    raise ValueError("Android calendar oracle returned incomplete data")
                with args.calendar.open("x", encoding="utf-8") as output:
                    json.dump(entries, output, allow_nan=False)
                    output.write("\n")
            if args.myanmar:
                raw_myanmar = subprocess.check_output(
                    [*adb, "shell", "dalvikvm", "-cp", f"{remote}:{apk}",
                     "MotaaNumericOracle", "myanmar"], text=True, timeout=30,
                )
                watat = [json.loads(line.removeprefix("myanmar_watat="))
                         for line in raw_myanmar.splitlines() if line.startswith("myanmar_watat=")]
                a0 = [json.loads(line.removeprefix("myanmar_a0="))
                      for line in raw_myanmar.splitlines() if line.startswith("myanmar_a0=")]
                if len(watat) != 31 or len(a0) != 20:
                    raise ValueError("Android Myanmar oracle returned incomplete data")
                with args.myanmar.open("x", encoding="utf-8") as output:
                    json.dump({"watat": watat, "a0": a0}, output, ensure_ascii=False, allow_nan=False)
                    output.write("\n")
            if args.rahu:
                raw = subprocess.check_output(
                    [*adb, "shell", "dalvikvm", "-cp", f"{remote}:{apk}",
                     "MotaaNumericOracle", "rahu"], text=True, timeout=30,
                )
                entries = [json.loads(line.removeprefix("rahu="))
                           for line in raw.splitlines() if line.startswith("rahu=")]
                if len(entries) != 81:
                    raise ValueError("Android Rahu oracle returned incomplete data")
                with args.rahu.open("x", encoding="utf-8") as output:
                    json.dump(entries, output, allow_nan=False)
                    output.write("\n")
            if args.delta_t:
                raw_delta = subprocess.check_output(
                    [*adb, "shell", "dalvikvm", "-cp", f"{remote}:{apk}",
                     "MotaaNumericOracle", "delta-t"], text=True, timeout=30,
                )
                entries = [json.loads(line.removeprefix("delta="))
                           for line in raw_delta.splitlines() if line.startswith("delta=")]
                if len(entries) != 2 * (301 * 12 + 1):
                    raise ValueError("Android Delta T oracle returned incomplete data")
                with args.delta_t.open("x", encoding="utf-8") as output:
                    json.dump(entries, output, separators=(",", ":"), allow_nan=False)
                    output.write("\n")
            if args.ayanamsha:
                raw = subprocess.check_output(
                    [*adb, "shell", "dalvikvm", "-cp", f"{remote}:{apk}",
                     "MotaaNumericOracle", "ayanamsha"], text=True, timeout=30,
                )
                entries = [json.loads(line.removeprefix("ayanamsha="))
                           for line in raw.splitlines() if line.startswith("ayanamsha=")]
                if len(entries) != 32 or any(len(row) != 10 for row in entries):
                    raise ValueError("Android ayanamsha oracle returned incomplete data")
                with args.ayanamsha.open("x", encoding="utf-8") as output:
                    json.dump(entries, output, separators=(",", ":"), allow_nan=False)
                    output.write("\n")
            if args.profile_database:
                capture_profile_database(adb, remote_db, args.profile_database)
        finally:
            subprocess.run([*adb, "shell", "rm", "-f", remote, remote_db, remote_db + "-journal"], check=False, timeout=15)
    with args.output.open("x", encoding="utf-8") as output:
        json.dump(records, output, separators=(",", ":"), allow_nan=False)
        output.write("\n")


if __name__ == "__main__":
    main()
