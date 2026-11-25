pub const GROUPS: [&str; 9] = [
    "Janma",
    "Sampat",
    "Vipat",
    "Kshema",
    "Pratyari",
    "Sadhaka",
    "Naidhana",
    "Mitra",
    "Parama Mitra",
];

pub const SPECIAL: [(&str, usize); 6] = [
    ("Janma", 1),
    ("Karma", 10),
    ("Sanghatika", 16),
    ("Samudayika", 18),
    ("Adhana", 19),
    ("Vinasha", 23),
];

pub fn mansion(longitude: f64) -> usize {
    ((longitude.rem_euclid(360.) / (360. / 27.)) as usize).min(26)
}

pub fn from_count(reference: usize, count: usize) -> usize {
    (reference + count - 1) % 27
}

pub fn group(reference: usize, target: usize) -> usize {
    ((target + 27 - reference) % 27) % 9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magha_to_revati_is_ninth_tara() {
        assert_eq!(GROUPS[group(9, 26)], "Parama Mitra");
        assert_eq!(GROUPS[group(26, 0)], "Sampat");
        assert_eq!(from_count(0, 16), 15);
        assert_eq!(from_count(0, 23), 22);
    }

    #[test]
    fn groups_partition_every_mansion_for_every_reference() {
        for reference in 0..27 {
            let mut counts = [0; 9];
            for target in 0..27 {
                counts[group(reference, target)] += 1;
            }
            assert_eq!(counts, [3; 9]);
        }
    }
}
