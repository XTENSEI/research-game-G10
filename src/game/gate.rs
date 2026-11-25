//! Kelp Forest / Castle Puzzle: The Lone Stone Throne Story (SCRIPT RES.pdf p. 15).
//! 14 interactive punctuation slots embedded inside the verbatim story text.
//! Players click any slot to assign punctuation, then click "Check Gate".
//! Multiple tries are allowed with gentle feedback showing how many slots fit.

/// Exact punctuation palette choices shown when editing a slot.
pub const PALETTE: &[(&str, &str)] = &[
    ("—", "Em-dash (—)"),
    (".", "Period (.)"),
    (",", "Comma (,)"),
    (":", "Colon (:)"),
    (";", "Semicolon (;)"),
    ("!", "Exclamation (!)"),
    ("-", "Hyphen (-)"),
    ("….", "Ellipsis + Period (….)"),
    ("…?", "Ellipsis + Question (…?)"),
    ("?", "Question (?)"),
];

pub struct GateSlot {
    /// Text fragment rendered *before* this interactive punctuation box.
    pub before: &'static str,
    /// Exact punctuation expected in this slot.
    pub answer: &'static str,
    /// Short grammar hint shown if missed on Check Gate.
    pub hint: &'static str,
}

/// Text printed *after* the 14th slot to complete the verbatim passage.
pub const TRAILING_TEXT: &str =
    " it was now able to teleport to different dimensions, leaving a permanent scar on the universe due to its use in the great civil war.";

/// Exact 14 blue-highlighted punctuation slots verified from SCRIPT RES.pdf p. 15.
pub const SLOTS: &[GateSlot] = &[
    GateSlot {
        before: "There was once an ancient city",
        answer: "—",
        hint: "Slot 1 sets off a dramatic modifier after 'city'. Try an em-dash.",
    },
    GateSlot {
        before: "prospering and standing",
        answer: ".",
        hint: "Slot 2 ends the opening sentence before 'It was ruled...'",
    },
    GateSlot {
        before: " It was ruled by a goddess",
        answer: ",",
        hint: "Slot 3 separates the participial phrase 'lacking her supernatural abilities'.",
    },
    GateSlot {
        before: " lacking her supernatural abilities. The goddess reigns over the city with three key values",
        answer: ":",
        hint: "Slot 4 introduces a three-item list after an independent clause.",
    },
    GateSlot {
        before: " integrity",
        answer: ",",
        hint: "Slot 5 separates items in the list of three values.",
    },
    GateSlot {
        before: " order",
        answer: ",",
        hint: "Slot 6 is the Oxford comma before 'and consistency'.",
    },
    GateSlot {
        before: " and consistency. The goddess doesn’t want anything above her, nor anything that disobeys her rule. One day",
        answer: ",",
        hint: "Slot 7 follows the introductory time phrase 'One day'.",
    },
    GateSlot {
        before: " artisans of the city created something extraordinary",
        answer: ":",
        hint: "Slot 8 introduces what the extraordinary creation is ('a portal...').",
    },
    GateSlot {
        before: " a portal they had made",
        answer: "….",
        hint: "Slot 9 trails off in awe before closing the sentence ('….').",
    },
    GateSlot {
        before: " “What is this sorcery that defies my order",
        answer: "…?",
        hint: "Slot 10 trails off into a sharp royal question ('…?').",
    },
    GateSlot {
        before: " Have it abolished",
        answer: "—",
        hint: "Slot 11 sharply interrupts before 'now!' with emphasis.",
    },
    GateSlot {
        before: "now",
        answer: "!",
        hint: "Slot 12 closes the queen's furious command inside the quote!",
    },
    GateSlot {
        before: "” the goddess spoke. The artisans, expecting praise, held a mutiny against the queen—driven by their newfound sorcery. This sparked a 200-year",
        answer: "-",
        hint: "Slot 13 joins the compound adjective '200-year-long' before 'war'.",
    },
    GateSlot {
        before: "long war that resulted in the portal evolving into something greater",
        answer: ";",
        hint: "Slot 14 links two closely related independent clauses without 'and'.",
    },
];

pub fn evaluate(picks: &[Option<usize>; 14]) -> (usize, Option<&'static str>) {
    let mut right = 0usize;
    let mut first_hint = None;
    for (i, slot) in SLOTS.iter().enumerate() {
        let matched = picks[i]
            .and_then(|pi| PALETTE.get(pi))
            .map(|(sym, _)| *sym == slot.answer)
            .unwrap_or(false);
        if matched {
            right += 1;
        } else if first_hint.is_none() {
            first_hint = Some(slot.hint);
        }
    }
    (right, first_hint)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_has_14_slots_and_reconstructs() {
        assert_eq!(SLOTS.len(), 14);
        let mut s = String::new();
        for slot in SLOTS {
            s.push_str(slot.before);
            s.push_str(slot.answer);
        }
        s.push_str(TRAILING_TEXT);
        assert!(s.starts_with("There was once an ancient city—prospering"));
        assert!(s.contains("200-year-long war"));
        assert!(s.contains("something greater; it was now able"));
    }
}
