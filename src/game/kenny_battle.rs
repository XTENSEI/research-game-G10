//! Kelp Forest Undertale-Style Tutorial Battle vs Kenny (SCRIPT RES.pdf pp. 1-2).
//! Combines bullet-box shell dodging with English/literacy quiz strikes.

#[derive(Clone, Copy)]
pub struct KennyQ {
    pub prompt: &'static str,
    pub choices: [&'static str; 3],
    pub correct: usize,
}

/// Full 3-question round for first duel; retries trim to first 2 questions per p. 2.
pub const KENNY_QUESTIONS: &[KennyQ] = &[
    KennyQ {
        prompt: "Kenny snaps: \"Alright land-kid, quick strike—which word completes this sentence in clean PAST TENSE?\"\n\"Earlier today, River ___ beside the glowing book.\"",
        choices: [
            "a) sleep",
            "b) slept",
            "c) sleeping",
        ],
        correct: 1,
    },
    KennyQ {
        prompt: "Kenny grins: \"Nice block! Now identify the ADVERB in:\n'Kenny scuttled swiftly past the tall kelp.'\"",
        choices: [
            "a) scuttled",
            "b) tall",
            "c) swiftly",
        ],
        correct: 2,
    },
    KennyQ {
        prompt: "Kenny's final test: \"What does the context word *prohibited* mean in:\n'Our King *prohibited* all weapons and fist fights across the realm'?\"",
        choices: [
            "a) Officially forbade by law",
            "b) Sold in market stalls",
            "c) Polished until shiny",
        ],
        correct: 0,
    },
];

pub fn questions_for(fewer_mode: bool) -> &'static [KennyQ] {
    if fewer_mode {
        &KENNY_QUESTIONS[..2]
    } else {
        KENNY_QUESTIONS
    }
}
