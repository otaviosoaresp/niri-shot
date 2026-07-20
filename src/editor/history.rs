use super::shapes::Shape;

#[derive(Clone)]
pub enum UndoEntry {
    Add {
        idx: usize,
        shape: Shape,
    },
    Modify {
        idx: usize,
        before: Shape,
        after: Shape,
    },
    Remove {
        idx: usize,
        shape: Shape,
    },
}

#[derive(Default)]
pub struct History {
    undo_stack: Vec<UndoEntry>,
    redo_stack: Vec<UndoEntry>,
}

impl History {
    /// Record a new edit. Any pending redo becomes unreachable, which is what
    /// keeps an undone shape from reappearing after an unrelated edit.
    pub fn push(&mut self, entry: UndoEntry) {
        self.undo_stack.push(entry);
        self.redo_stack.clear();
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, shapes: &mut Vec<Shape>) -> bool {
        let Some(entry) = self.undo_stack.pop() else {
            return false;
        };

        match &entry {
            UndoEntry::Add { idx, .. } => {
                if *idx < shapes.len() {
                    shapes.remove(*idx);
                }
            }
            UndoEntry::Modify { idx, before, .. } => {
                if let Some(shape) = shapes.get_mut(*idx) {
                    *shape = before.clone();
                }
            }
            UndoEntry::Remove { idx, shape } => {
                shapes.insert((*idx).min(shapes.len()), shape.clone());
            }
        }

        self.redo_stack.push(entry);
        true
    }

    pub fn redo(&mut self, shapes: &mut Vec<Shape>) -> bool {
        let Some(entry) = self.redo_stack.pop() else {
            return false;
        };

        match &entry {
            UndoEntry::Add { idx, shape } => {
                shapes.insert((*idx).min(shapes.len()), shape.clone());
            }
            UndoEntry::Modify { idx, after, .. } => {
                if let Some(shape) = shapes.get_mut(*idx) {
                    *shape = after.clone();
                }
            }
            UndoEntry::Remove { idx, .. } => {
                if *idx < shapes.len() {
                    shapes.remove(*idx);
                }
            }
        }

        self.undo_stack.push(entry);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::shapes::{Color, ShapeType};

    fn shape_at(x: f64) -> Shape {
        Shape {
            shape_type: ShapeType::Rectangle,
            start_x: x,
            start_y: 0.0,
            end_x: x + 10.0,
            end_y: 10.0,
            color: Color::new(1.0, 0.0, 0.0, 1.0),
            stroke_width: 2.0,
            ..Default::default()
        }
    }

    /// Build a document plus the history that would have produced it by drawing.
    fn drawn(count: usize) -> (Vec<Shape>, History) {
        let mut shapes = Vec::new();
        let mut history = History::default();
        for i in 0..count {
            let shape = shape_at(i as f64 * 100.0);
            shapes.push(shape.clone());
            history.push(UndoEntry::Add { idx: i, shape });
        }
        (shapes, history)
    }

    fn move_shape(shapes: &mut [Shape], history: &mut History, idx: usize, dx: f64) {
        let before = shapes[idx].clone();
        shapes[idx].translate(dx, 0.0);
        let after = shapes[idx].clone();
        history.push(UndoEntry::Modify { idx, before, after });
    }

    fn xs(shapes: &[Shape]) -> Vec<f64> {
        shapes.iter().map(|s| s.start_x).collect()
    }

    #[test]
    fn undo_reverts_a_move_instead_of_deleting_the_shape() {
        let (mut shapes, mut history) = drawn(1);
        move_shape(&mut shapes, &mut history, 0, 50.0);
        assert_eq!(xs(&shapes), vec![50.0]);

        history.undo(&mut shapes);
        assert_eq!(
            xs(&shapes),
            vec![0.0],
            "undo restores the position and keeps the shape"
        );

        history.redo(&mut shapes);
        assert_eq!(xs(&shapes), vec![50.0], "redo re-applies the move");
    }

    #[test]
    fn undo_after_delete_restores_the_original_index() {
        let (mut shapes, mut history) = drawn(3);

        let removed = shapes.remove(1);
        history.push(UndoEntry::Remove {
            idx: 1,
            shape: removed,
        });
        assert_eq!(xs(&shapes), vec![0.0, 200.0]);

        history.undo(&mut shapes);
        assert_eq!(
            xs(&shapes),
            vec![0.0, 100.0, 200.0],
            "the deleted shape returns to its own slot, not the end"
        );
    }

    #[test]
    fn redo_cannot_resurrect_a_shape_after_an_intervening_edit() {
        let (mut shapes, mut history) = drawn(2);

        history.undo(&mut shapes);
        assert_eq!(xs(&shapes), vec![0.0], "second shape undone");

        move_shape(&mut shapes, &mut history, 0, 5.0);
        history.redo(&mut shapes);

        assert_eq!(
            xs(&shapes),
            vec![5.0],
            "the new edit invalidates the redo stack"
        );
    }

    #[test]
    fn redo_after_a_delete_does_not_bring_the_shape_back() {
        let (mut shapes, mut history) = drawn(1);

        let removed = shapes.remove(0);
        history.push(UndoEntry::Remove {
            idx: 0,
            shape: removed,
        });

        history.redo(&mut shapes);
        assert!(
            shapes.is_empty(),
            "delete is not an undo; redo must not restore it"
        );
    }

    #[test]
    fn a_mixed_history_round_trips_exactly() {
        let (mut shapes, mut history) = drawn(3);

        let removed = shapes.remove(0);
        history.push(UndoEntry::Remove {
            idx: 0,
            shape: removed,
        });
        move_shape(&mut shapes, &mut history, 1, 7.0);

        let final_state = xs(&shapes);
        assert_eq!(final_state, vec![100.0, 207.0]);

        while history.undo(&mut shapes) {}
        assert!(
            shapes.is_empty(),
            "unwinding the full history empties the document"
        );

        while history.redo(&mut shapes) {}
        assert_eq!(
            xs(&shapes),
            final_state,
            "replaying the full history reproduces the exact state"
        );
    }

    #[test]
    fn undo_and_redo_on_an_empty_history_are_no_ops() {
        let mut shapes = Vec::new();
        let mut history = History::default();

        assert!(!history.undo(&mut shapes));
        assert!(!history.redo(&mut shapes));
        assert!(shapes.is_empty());
    }
}
