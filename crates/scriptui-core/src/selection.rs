#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Selection {
    pub anchor: usize,
    pub head: usize,
}

impl Selection {
    pub const fn point(offset: usize) -> Self {
        Self {
            anchor: offset,
            head: offset,
        }
    }

    pub const fn new(anchor: usize, head: usize) -> Self {
        Self { anchor, head }
    }

    pub const fn start(self) -> usize {
        if self.anchor <= self.head {
            self.anchor
        } else {
            self.head
        }
    }

    pub const fn end(self) -> usize {
        if self.anchor >= self.head {
            self.anchor
        } else {
            self.head
        }
    }

    pub const fn is_empty(self) -> bool {
        self.anchor == self.head
    }

    pub const fn len(self) -> usize {
        self.end() - self.start()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionSet {
    ranges: Vec<Selection>,
    primary: usize,
}

impl SelectionSet {
    pub fn point(offset: usize) -> Self {
        Self {
            ranges: vec![Selection::point(offset)],
            primary: 0,
        }
    }

    pub fn from_ranges(ranges: Vec<Selection>) -> Self {
        Self::from_ranges_with_primary(ranges, 0)
    }

    pub fn from_ranges_with_primary(ranges: Vec<Selection>, primary: usize) -> Self {
        if ranges.is_empty() {
            return Self::point(0);
        }
        let mut set = Self { ranges, primary };
        set.normalize();
        set
    }

    pub fn ranges(&self) -> &[Selection] {
        &self.ranges
    }

    pub fn primary(&self) -> Selection {
        self.ranges
            .get(self.primary)
            .copied()
            .unwrap_or(Selection::point(0))
    }

    pub fn primary_index(&self) -> usize {
        self.primary
    }

    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    pub fn clamp(&mut self, max_char_len: usize) {
        for sel in &mut self.ranges {
            sel.anchor = sel.anchor.min(max_char_len);
            sel.head = sel.head.min(max_char_len);
        }
        self.normalize();
    }

    pub fn collapse_to_primary(&mut self) {
        let head = self.primary().head;
        self.ranges.clear();
        self.ranges.push(Selection::point(head));
        self.primary = 0;
    }

    fn normalize(&mut self) {
        if self.ranges.is_empty() {
            self.ranges.push(Selection::point(0));
            self.primary = 0;
            return;
        }

        let primary_idx = self.primary.min(self.ranges.len() - 1);
        let mut tagged: Vec<(Selection, bool)> = self
            .ranges
            .drain(..)
            .enumerate()
            .map(|(idx, sel)| (sel, idx == primary_idx))
            .collect();

        tagged.sort_by_key(|(sel, _)| (sel.start(), sel.end()));

        let mut merged: Vec<Selection> = Vec::with_capacity(tagged.len());
        let mut new_primary = 0usize;
        for (next, is_primary) in tagged {
            if let Some(last) = merged.last_mut() {
                let overlaps = next.start() < last.end()
                    || (next.start() == last.end() && (last.is_empty() || next.is_empty()));
                if overlaps {
                    let is_backward = if last.is_empty() {
                        next.head < next.anchor
                    } else {
                        last.head < last.anchor
                    };
                    let start = last.start().min(next.start());
                    let end = last.end().max(next.end());
                    *last = if is_backward {
                        Selection::new(end, start)
                    } else {
                        Selection::new(start, end)
                    };
                    if is_primary {
                        new_primary = merged.len() - 1;
                    }
                    continue;
                }
            }
            if is_primary {
                new_primary = merged.len();
            }
            merged.push(next);
        }

        self.ranges = merged;
        self.primary = if self.ranges.is_empty() {
            0
        } else {
            new_primary
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_start_end_and_empty() {
        let pt = Selection::point(4);
        assert!(pt.is_empty());
        assert_eq!(pt.len(), 0);
        assert_eq!(pt.start(), 4);
        assert_eq!(pt.end(), 4);

        let forward = Selection::new(2, 7);
        assert!(!forward.is_empty());
        assert_eq!(forward.start(), 2);
        assert_eq!(forward.end(), 7);
        assert_eq!(forward.len(), 5);

        let backward = Selection::new(9, 3);
        assert!(!backward.is_empty());
        assert_eq!(backward.start(), 3);
        assert_eq!(backward.end(), 9);
        assert_eq!(backward.len(), 6);
    }

    #[test]
    fn selection_set_merges_overlaps_and_sorts() {
        let set = SelectionSet::from_ranges(vec![
            Selection::new(10, 12),
            Selection::new(2, 6),
            Selection::new(4, 8),
        ]);
        assert_eq!(set.len(), 2);
        assert_eq!(set.ranges()[0], Selection::new(2, 8));
        assert_eq!(set.ranges()[1], Selection::new(10, 12));
    }

    #[test]
    fn selection_set_clamp_and_collapse() {
        let mut set =
            SelectionSet::from_ranges(vec![Selection::new(5, 20), Selection::new(15, 25)]);
        set.clamp(10);
        assert_eq!(set.len(), 1);
        assert_eq!(set.ranges()[0], Selection::new(5, 10));
        set.collapse_to_primary();
        assert_eq!(set.primary(), Selection::point(10));
    }

    #[test]
    fn normalize_remaps_primary_across_sort_and_merge() {
        // Primary starts at index 0 with a higher offset; after sorting it moves to index 1.
        let sorted = SelectionSet::from_ranges(vec![Selection::new(10, 12), Selection::new(2, 4)]);
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted.primary(), Selection::new(10, 12));
        assert_eq!(sorted.primary_index(), 1);

        // Primary starts at index 0 (12..16), sorts after 2..4, and merges into 10..16 at index 1.
        let merged = SelectionSet::from_ranges_with_primary(
            vec![
                Selection::new(12, 16),
                Selection::new(2, 4),
                Selection::new(10, 14),
            ],
            0,
        );
        assert_eq!(merged.len(), 2);
        assert_eq!(merged.ranges()[0], Selection::new(2, 4));
        assert_eq!(merged.ranges()[1], Selection::new(10, 16));
        assert_eq!(merged.primary(), Selection::new(10, 16));
        assert_eq!(merged.primary_index(), 1);
    }
}
