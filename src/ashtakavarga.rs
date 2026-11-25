// Contributor order: Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Ascendant.
// Varahamihira's Brihat Jataka convention. Houses count inclusively from each
// contributor's natal sign. These are the unreduced benefic point tables.
const HOUSES: [[&[u8]; 8]; 7] = [
    [
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 6, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 5, 6, 9, 10, 11, 12],
        &[5, 6, 9, 11],
        &[6, 7, 12],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[3, 4, 6, 10, 11, 12],
    ],
    [
        &[3, 6, 7, 8, 10, 11],
        &[1, 3, 6, 7, 10, 11],
        &[2, 3, 5, 6, 9, 10, 11],
        &[1, 3, 4, 5, 7, 8, 10, 11],
        &[1, 4, 7, 8, 10, 11, 12],
        &[3, 4, 5, 7, 9, 10, 11],
        &[3, 5, 6, 11],
        &[3, 6, 10, 11],
    ],
    [
        &[3, 5, 6, 10, 11],
        &[3, 6, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 5, 6, 11],
        &[6, 10, 11, 12],
        &[6, 8, 11, 12],
        &[1, 4, 7, 8, 9, 10, 11],
        &[1, 3, 6, 10, 11],
    ],
    [
        &[5, 6, 9, 11, 12],
        &[2, 4, 6, 8, 10, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[1, 3, 5, 6, 9, 10, 11, 12],
        &[6, 8, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 11],
        &[1, 2, 4, 7, 8, 9, 10, 11],
        &[1, 2, 4, 6, 8, 10, 11],
    ],
    [
        &[1, 2, 3, 4, 7, 8, 9, 10, 11],
        &[2, 5, 7, 9, 11],
        &[1, 2, 4, 7, 8, 10, 11],
        &[1, 2, 4, 5, 6, 9, 10, 11],
        &[1, 2, 3, 4, 7, 8, 10, 11],
        &[2, 5, 6, 9, 10, 11],
        &[3, 5, 6, 12],
        &[1, 2, 4, 5, 6, 7, 9, 10, 11],
    ],
    [
        &[8, 11, 12],
        &[1, 2, 3, 4, 5, 8, 9, 11, 12],
        &[3, 5, 6, 9, 11, 12],
        &[3, 5, 6, 9, 11],
        &[5, 8, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9, 10, 11],
        &[3, 4, 5, 8, 9, 10, 11],
        &[1, 2, 3, 4, 5, 8, 9, 11],
    ],
    [
        &[1, 2, 4, 7, 8, 10, 11],
        &[3, 6, 11],
        &[3, 5, 6, 10, 11, 12],
        &[6, 8, 9, 10, 11, 12],
        &[5, 6, 11, 12],
        &[6, 11, 12],
        &[3, 5, 6, 11],
        &[1, 3, 4, 6, 10, 11],
    ],
];

pub fn calculate(signs: [usize; 8]) -> ([[u8; 12]; 7], [u8; 12]) {
    let individual = HOUSES.map(|rows| {
        let mut points = [0; 12];
        for (sign, houses) in signs.into_iter().zip(rows) {
            for &house in houses {
                points[(sign + usize::from(house) - 1) % 12] += 1;
            }
        }
        points
    });
    let total = std::array::from_fn(|sign| individual.iter().map(|row| row[sign]).sum());
    (individual, total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_full_published_libra_ascendant_example() {
        let (individual, total) = calculate([3, 1, 0, 4, 3, 3, 8, 6]);
        assert_eq!(
            individual,
            [
                [3, 4, 3, 6, 3, 2, 5, 2, 6, 6, 3, 5],
                [4, 7, 3, 3, 3, 3, 5, 3, 4, 4, 6, 4],
                [3, 4, 4, 4, 2, 2, 5, 2, 6, 2, 2, 3],
                [2, 6, 5, 5, 5, 2, 6, 4, 6, 4, 4, 5],
                [7, 6, 3, 4, 5, 4, 4, 6, 2, 6, 5, 4],
                [5, 5, 5, 3, 5, 4, 4, 3, 4, 3, 6, 5],
                [3, 5, 4, 4, 3, 1, 4, 1, 3, 4, 3, 4],
            ]
        );
        assert_eq!(total, [27, 37, 27, 29, 26, 18, 33, 21, 31, 29, 29, 30]);
    }

    #[test]
    fn totals_stay_fixed_across_sign_changes() {
        let (individual, total) = calculate([11, 3, 5, 1, 7, 2, 4, 9]);
        assert_eq!(
            individual.map(|row| row.into_iter().map(u16::from).sum::<u16>()),
            [48, 49, 39, 54, 56, 52, 39]
        );
        assert_eq!(
            total.iter().map(|&points| u16::from(points)).sum::<u16>(),
            337
        );
    }
}
