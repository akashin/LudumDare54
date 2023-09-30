use core::fmt;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum BuildingType {
    House,  // 1
    Trash,  // T
    Hermit, // H
}

impl BuildingType {
    pub fn to_char(&self) -> u8 {
        match self {
            BuildingType::House => b'1',
            BuildingType::Trash => b'T',
            BuildingType::Hermit => b'H',
        }
    }

    pub fn from_char(c: u8) -> BuildingType {
        match c {
            b'1' => BuildingType::House,
            b'T' => BuildingType::Trash,
            b'H' => BuildingType::Hermit,
            _ => panic!("Unknown building type: {}", c),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CellType {
    Grass,
    Hole,
}

impl CellType {
    pub fn to_char(&self) -> char {
        match self {
            CellType::Grass => 'g',
            CellType::Hole => 'x',
        }
    }
}

#[derive(Debug)]
pub struct Level {
    pub building_count: HashMap<BuildingType, usize>,
    field: Vec<Vec<CellType>>,
}

impl Level {
    pub fn rows(&self) -> usize {
        self.field.len()
    }

    pub fn columns(&self) -> usize {
        self.field[0].len()
    }
}

impl fmt::Display for Level {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..self.rows() {
            for column in 0..self.columns() {
                write!(formatter, "{}", self.field[row][column].to_char())?
            }
            write!(formatter, "\n")?
        }
        write!(formatter, "{:?}", self.building_count)
    }
}

pub fn field_from_size(rows: usize, columns: usize) -> Vec<Vec<CellType>> {
    vec![vec![CellType::Grass; columns]; rows]
}

pub struct Placement {
    building: BuildingType,
    row: usize,
    column: usize,
}

pub struct Solution {
    placements: Vec<Placement>,
}

pub fn parse_solution(s: Vec<&str>) -> Solution {
    let mut solution = Solution {
        placements: Vec::new(),
    };
    for row in 0..s.len() {
        for column in 0..s[row].len() {
            let c = s[row].as_bytes()[column];
            // Skip cells with background objects.
            if [b'.', b'g', b'x'].contains(&c) {
                continue;
            }
            solution.placements.push(Placement {
                building: BuildingType::from_char(c),
                row,
                column,
            })
        }
    }
    solution
}

const DROW: [i32; 4] = [1, 0, -1, 0];
const DCOL: [i32; 4] = [1, 0, -1, 0];

#[derive(Debug)]
enum ViolationType {
    NoGrass,
}

#[derive(Debug)]
pub struct PlacementViolation {
    building_index: usize,
    violation: ViolationType,
}

#[derive(Debug)]
pub struct ValidationResult {
    building_missing: bool,
    placement_violations: Vec<PlacementViolation>,
}

pub fn validate_solution(solution: &Solution, level: &Level) -> ValidationResult {
    let mut placement_violations = Vec::new();

    // Check that we have the right count of each building.
    let mut building_count = HashMap::new();
    for placement in &solution.placements {
        *building_count.entry(placement.building).or_insert(0) += 1;
    }
    if level.building_count != building_count {
        return ValidationResult {
            building_missing: true,
            placement_violations,
        };
    }

    // Check that houses have grass nearby.
    for (index, placement) in solution.placements.iter().enumerate() {
        if matches!(placement.building, BuildingType::House) {
            let mut found_grass = false;
            for d in 0..4 {
                let nrow = placement.row as i32 + DROW[d];
                let ncol = placement.column as i32 + DCOL[d];
                if nrow < 0
                    || nrow >= level.rows() as i32
                    || ncol < 0
                    || ncol >= level.columns() as i32
                {
                    continue;
                }

                let nrow = nrow as usize;
                let ncol = ncol as usize;
                if level.field[nrow][ncol] == CellType::Grass {
                    found_grass = true;
                    break;
                }
            }

            if !found_grass {
                placement_violations.push(PlacementViolation {
                    building_index: index,
                    violation: ViolationType::NoGrass,
                })
            }
        }
    }

    // Check that hermits are on the edges.
    // Check that houses don't have trash next to them.
    return ValidationResult {
        building_missing: false,
        placement_violations,
    };
}

#[rustfmt::skip]
pub fn first_level() -> (Level, Solution) {
    (
        Level {
            building_count: HashMap::from([(BuildingType::House, 5), (BuildingType::Trash, 1)]),
            field: field_from_size(3, 3),
        },
        parse_solution(vec![
           "1gT", 
           "11g",
           "g11",
        ]),
    )
}

#[rustfmt::skip]
pub fn second_level() -> (Level, Solution) {
    (
        Level {
            building_count: HashMap::from([(BuildingType::House, 4), (BuildingType::Hermit, 4)]),
            field: field_from_size(3, 3),
        },
        parse_solution(vec![
           "H1H", 
           "1g1",
           "H1H",
        ]),
    )
}
