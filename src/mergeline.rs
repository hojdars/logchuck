use core::fmt;

#[cfg(test)]
mod test;

#[derive(Debug, Clone)]
pub struct Line {
    pub timestamp: u64,
    pub source_file: String,
    pub index: u64,
}

pub fn merge(left: &Vec<Line>, right: &Vec<Line>) -> Vec<Line> {
    let mut index_left: usize = 0; // maximum = left.len()
    let mut index_right: usize = 0; // maximum = right.len()

    let mut result: Vec<Line> = Vec::new();

    loop {
        if index_left == left.len() || index_right == right.len() {
            break;
        }

        if left[index_left] <= right[index_right] {
            result.push(left[index_left].clone());
            index_left += 1;
        } else {
            assert!(left[index_left] > right[index_right]);
            result.push(right[index_right].clone());
            index_right += 1;
        }
    }

    if index_left < left.len() {
        loop {
            if index_left == left.len() {
                break;
            }

            result.push(left[index_left].clone());
            index_left += 1;
        }
    }

    if index_right < right.len() {
        loop {
            if index_right == right.len() {
                break;
            }

            result.push(right[index_right].clone());
            index_right += 1;
        }
    }

    result
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Line({}, {}, {})",
            self.timestamp, self.source_file, self.index
        )
    }
}

impl std::cmp::PartialEq for Line {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl std::cmp::Eq for Line {}

impl std::cmp::PartialOrd for Line {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for Line {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        std::cmp::Ord::cmp(&self.timestamp, &other.timestamp)
    }
}
