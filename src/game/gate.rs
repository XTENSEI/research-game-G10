//! The Lone Stone Story of Throne Gate. Passage verbatim from the script,
//! with the boxed punctuation slots marked as (index, expected mark).

pub struct GateSlot {
    /// Word offset in the passage where the slot sits (for rendering).
    pub after_word: usize,
    /// The correct punctuation mark.
    pub answer: &'static str,
    /// Wrong options shown alongside.
    pub decoys: &'static [&'static str],
}

pub struct GatePuzzle {
    pub passage: &'static str,
    pub slots: &'static [GateSlot],
}

pub const THRONE_GATE: GatePuzzle = GatePuzzle {
    passage: "There was once an ancient city prospering and standing It was ruled by a goddess lacking her supernatural abilities The goddess reigns over the city with three key values integrity order and consistency The goddess doesn't want anything above her nor anything that disobeys her rule One day artisans of the city created something extraordinary a portal they had made What is this sorcery that defies my order Have it abolished now the goddess spoke The artisans expecting praise held a mutiny against the queen using their newfound sorcery This sparked a 200 year long war that resulted in the portal evolving into something greater it was now able to teleport to different dimensions leaving a permanent scar on the universe due to its use in the war",
    slots: &[
        GateSlot { after_word: 7, answer: "\u{2014}", decoys: &["-", ";", ":"] },        // city\u{2014}prospering
        GateSlot { after_word: 9, answer: ".", decoys: &[",", ";", "!"] },               // standing.
        GateSlot { after_word: 17, answer: ",", decoys: &[".", ":", "-"] },              // goddess,
        GateSlot { after_word: 30, answer: ":", decoys: &[";", ",", "."] },              // values:
        GateSlot { after_word: 32, answer: ",", decoys: &[".", ":", ";"] },              // integrity,
        GateSlot { after_word: 34, answer: ",", decoys: &[".", ":", ";"] },              // order,
        GateSlot { after_word: 53, answer: ":", decoys: &[";", ",", "."] },              // extraordinary:
        GateSlot { after_word: 58, answer: "\u{2026}", decoys: &[".", "!", "?"] },       // made\u{2026}
        GateSlot { after_word: 66, answer: "\u{2026}", decoys: &[".", "!", "?"] },       // order\u{2026}
        GateSlot { after_word: 70, answer: "\u{2014}", decoys: &["-", ";", ":"] },       // abolished\u{2014}now
        GateSlot { after_word: 72, answer: "!", decoys: &[".", "?", ";"] },              // now!
        GateSlot { after_word: 79, answer: "-", decoys: &["\u{2014}", ":", ";"] },       // 200-year
        GateSlot { after_word: 97, answer: ";", decoys: &[".", ",", ":"] },              // greater;
    ],
};
