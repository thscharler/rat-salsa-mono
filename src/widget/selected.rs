use crate::util::{next_opt, prev_opt};
use log::debug;
use std::collections::HashSet;
use std::fmt::Debug;
use std::mem;

/// Trait for using a selection.
///
pub trait Selection {
    /// Is selected.
    fn is_selected(&self, n: usize) -> bool;

    /// Selection lead.
    fn lead_selection(&self) -> Option<usize>;
}

// -----------------------------------------------------------------------
// -----------------------------------------------------------------------

/// NoSelection
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NoSelection;

impl Selection for NoSelection {
    fn is_selected(&self, _n: usize) -> bool {
        false
    }

    fn lead_selection(&self) -> Option<usize> {
        None
    }
}

// -----------------------------------------------------------------------
// -----------------------------------------------------------------------

/// Single element selection.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct SingleSelection {
    pub selected: Option<usize>,
}

impl Selection for SingleSelection {
    fn is_selected(&self, n: usize) -> bool {
        self.selected == Some(n)
    }

    fn lead_selection(&self) -> Option<usize> {
        self.selected
    }
}

impl SingleSelection {
    pub fn new() -> SingleSelection {
        SingleSelection { selected: None }
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn select(&mut self, select: Option<usize>) {
        self.selected = select;
    }

    pub fn select_clamped(&mut self, select: usize, max: usize) {
        if select <= max {
            self.selected = Some(select);
        }
    }

    pub fn next(&mut self, n: usize, max: usize) {
        self.selected = next_opt(self.selected, n, max);
    }

    pub fn prev(&mut self, n: usize) {
        self.selected = prev_opt(self.selected, n);
    }
}

// -----------------------------------------------------------------------
// -----------------------------------------------------------------------

/// List selection
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct SetSelection {
    pub anchor: Option<usize>,
    pub lead: Option<usize>,
    pub selected: HashSet<usize>,
}

impl Selection for SetSelection {
    fn is_selected(&self, n: usize) -> bool {
        if let Some(mut anchor) = self.anchor {
            if let Some(mut lead) = self.lead {
                if lead < anchor {
                    mem::swap(&mut lead, &mut anchor);
                }

                if n >= anchor && n <= lead {
                    return true;
                }
            }
        } else {
            if let Some(lead) = self.lead {
                if n == lead {
                    return true;
                }
            }
        }

        self.selected.contains(&n)
    }

    fn lead_selection(&self) -> Option<usize> {
        self.lead
    }
}

impl SetSelection {
    pub fn new() -> SetSelection {
        SetSelection {
            anchor: None,
            lead: None,
            selected: HashSet::new(),
        }
    }

    fn extend(&mut self, extend: bool) {
        if extend {
            if self.anchor.is_none() {
                self.anchor = self.lead;
            }
        } else {
            self.anchor = None;
            self.selected.clear();
        }
    }

    pub fn next(&mut self, n: usize, max: usize, extend: bool) {
        self.extend(extend);
        self.lead = next_opt(self.lead, n, max);
    }

    pub fn prev(&mut self, n: usize, extend: bool) {
        self.extend(extend);
        self.lead = prev_opt(self.lead, n);
    }

    pub fn set_lead(&mut self, lead: Option<usize>, extend: bool) {
        self.extend(extend);
        self.lead = lead;
    }

    pub fn set_lead_clamped(&mut self, lead: usize, max: usize, extend: bool) {
        if lead <= max {
            self.extend(extend);
            self.lead = Some(lead);
        }
    }

    pub fn lead(&self) -> Option<usize> {
        self.lead
    }

    pub fn anchor(&self) -> Option<usize> {
        self.anchor
    }

    pub fn transfer_lead_anchor(&mut self) {
        Self::fill(self.anchor, self.lead, &mut self.selected);
        self.anchor = None;
        self.lead = None;
    }

    fn fill(anchor: Option<usize>, lead: Option<usize>, selection: &mut HashSet<usize>) {
        if let Some(mut anchor) = anchor {
            if let Some(mut lead) = lead {
                if lead < anchor {
                    mem::swap(&mut lead, &mut anchor);
                }

                for n in anchor..=lead {
                    selection.insert(n);
                }
            }
        } else {
            if let Some(lead) = lead {
                selection.insert(lead);
            }
        }
    }

    pub fn clear(&mut self) {
        self.anchor = None;
        self.lead = None;
        self.selected.clear();
    }

    pub fn add(&mut self, idx: usize) {
        self.selected.insert(idx);
    }

    pub fn remove(&mut self, idx: usize) {
        self.selected.remove(&idx);
    }
}

// -----------------------------------------------------------------------
// -----------------------------------------------------------------------
