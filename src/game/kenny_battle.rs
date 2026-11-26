//! Kelp Forest Undertale-Style Tutorial Battle vs Kenny (SCRIPT RES.pdf pp. 1-2).
//! Combines bullet-box shell dodging with English/literacy quiz strikes.

#[derive(Clone, Copy)]
pub struct KennyQ {
    pub prompt: &'static str,
    pub choices: [&'static str; 3],
    pub correct: usize,
}

/// Full 3-question round combining conversation recall & grammar; retries trim to first 2.
pub const KENNY_QUESTIONS: &[KennyQ] = &[
    KennyQ {
        prompt: "Kenny snaps: \"First check, were you paying attention?! Why does everyone out here settle duels with riddles instead of fists?\"",
        choices: [
            "a) Nobody knows how to craft spears underwater",
            "b) The King prohibited weapons and fighting in duels",
            "c) Only Queen Crab is allowed to carry weapons",
        ],
        correct: 1,
    },
    KennyQ {
        prompt: "Kenny leans in: \"Nice! Now pick the clean PAST TENSE verb:\n'Earlier tonight, River ___ beside the glowing book.'\"",
        choices: [
            "a) sleep",
            "b) sleeping",
            "c) slept",
        ],
        correct: 2,
    },
    KennyQ {
        prompt: "Kenny's final test: \"What word acts as the ADVERB in:\n'Kenny scuttled swiftly past the tall kelp.'\"",
        choices: [
            "a) swiftly",
            "b) tall",
            "c) scuttled",
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
