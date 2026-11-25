//! Biome 2 Sea Anglers Rap Battle, Biome 3 Temple of The Figurate Algae,
//! and The Rainbow Road + Magic Fire Genie poetry trial (SCRIPT RES.pdf pp. 5-9, 17-19, 21).

// ---------------------------------------------------------------------------
// Sea Anglers Clean Rap Battle (Jevil Seamountain Range)
// ---------------------------------------------------------------------------

pub struct RapBar {
    pub prompt: &'static str,
    pub rhyming_word: &'static str,
    pub decoys: [&'static str; 2],
}

pub const RAP_BARS: &[RapBar] = &[
    RapBar {
        prompt: "Angler Trio: \"Three deep-sea MC's blocking up the mountain bend,\nDrop a verse right now or hit a dead ___!\"",
        rhyming_word: "end",
        decoys: ["splash", "shell"],
    },
    RapBar {
        prompt: "River: \"No weapons down here, just syntax and rhyme,\nI've decoded harder riddles every single ___!\"",
        rhyming_word: "time",
        decoys: ["tide", "reef"],
    },
    RapBar {
        prompt: "Tow-Tou: \"Brass flipper spinning while we climb the stone hill,\nWait until this landlubber proves they got ___!\"",
        rhyming_word: "skill",
        decoys: ["glow", "luck"],
    },
    RapBar {
        prompt: "River: \"Got the pearl, got the berries, kept my diction super bright.\nStand aside, Angler Trio! Watch me claim the mountain ___!\"",
        rhyming_word: "height",
        decoys: ["deep", "foam"],
    },
];

// ---------------------------------------------------------------------------
// Temple of The Figurate Algae (Somber Slums Alchemy Trial)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct AlgaeTrial {
    pub chamber: &'static str,
    pub riddle_text: &'static str,
    pub sequence_hint: &'static str,
    pub choices: [&'static str; 4],
    pub correct: usize,
    pub infuse_note: &'static str,
}

pub const ALGAE_TRIALS: &[AlgaeTrial] = &[
    AlgaeTrial {
        chamber: "Chamber I: Triangular Spore Lattice",
        riddle_text: "\"Like a lighthouse beacon cutting velvet night...\"\nIdentify the device AND next triangular algae count: 1, 3, 6, 10, ?",
        sequence_hint: "Add +2, +3, +4, +5 to each glowing frond.",
        choices: [
            "Simile + 15",
            "Hyperbole + 14",
            "Onomatopoeia + 12",
            "Oxymoron + 16",
        ],
        correct: 0,
        infuse_note: "Queen Crab's *Pearl of Clarity* dissolves into emerald algae!",
    },
    AlgaeTrial {
        chamber: "Chamber II: Square Coral Bloom",
        riddle_text: "\"The silent trench whispered old secrets to our oars.\"\nIdentify the device AND next square crystal count: 1, 4, 9, 16, ?",
        sequence_hint: "Square numbers: 1^2, 2^2, 3^2, 4^2, 5^2.",
        choices: [
            "Alliteration + 20",
            "Personification + 25",
            "Irony + 24",
            "Allusion + 21",
        ],
        correct: 1,
        infuse_note: "The *Blue Seashroom & Tentacle Anemone* gleam silver-bright!",
    },
    AlgaeTrial {
        chamber: "Chamber III: Seal of The Elixir",
        riddle_text: "\"Words are lanterns carried across uncharted seas.\"\nIdentify the device AND closing pentagonal stone count: 1, 5, 12, 22, ?",
        sequence_hint: "Pentagonal differences grow by +3 each step: +4, +7, +10, +13.",
        choices: [
            "Simile + 30",
            "Pun + 32",
            "Metaphor + 35",
            "Euphemism + 28",
        ],
        correct: 2,
        infuse_note: "True Seaberry completes the brew: the *Elixir of Literacy* is whole!!",
    },
];

// ---------------------------------------------------------------------------
// The Rainbow Road: Falling Stars + Torch Slabs + Magic Fire Genie Poems
// ---------------------------------------------------------------------------

pub const STAR_TARGET: u32 = 14;
pub const DYING_PENALTY: u32 = 3;
pub const LIVES: u32 = 3;

#[derive(Clone, Copy)]
pub struct PoemQ {
    pub prompt: &'static str,
    pub choices: [&'static str; 4],
    pub correct: usize,
    pub feedback: &'static str,
}

#[derive(Clone, Copy)]
pub struct Poem {
    pub title: &'static str,
    pub author: &'static str,
    pub lines: &'static [&'static str],
    pub questions: &'static [PoemQ],
}

/// Primary poem from SCRIPT RES.pdf pp. 17-18, plus alternates shown if retrying!
pub const POEMS: &[Poem] = &[
    Poem {
        title: "Because I could not stop for Death",
        author: "Emily Dickinson (1890)",
        lines: &[
            "Because I could not stop for Death -",
            "He kindly stopped for me -",
            "The Carriage held but just Ourselves -",
            "And Immortality.",
            "",
            "We slowly drove - He knew no haste",
            "And I had put away",
            "My labor and my leisure too,",
            "For His Civility -",
            "",
            "We passed the School, where Children strove",
            "At Recess - in the Ring -",
            "We passed the Fields of Gazing Grain -",
            "We passed the Setting Sun -",
            "",
            "Or rather - He passed Us -",
            "The Dews drew quivering and Chill -",
            "Since then - 'tis Centuries - and yet",
            "Feels shorter than the Day",
            "I first surmised the Horses' Heads",
            "Were toward Eternity -",
        ],
        questions: &[
            PoemQ {
                prompt: "What is the central message of Dickinson's poem?",
                choices: [
                    "Death is inevitable in our lives; we don't decide when it comes either",
                    "Hard physical labor keeps mortality far away",
                    "Childhood school days should never end",
                    "Carriage travel across countryside ruins takes years",
                ],
                correct: 0,
                feedback: "Spot on! Mortality arrives on its own schedule, outside our control.",
            },
            PoemQ {
                prompt: "How does the poem depict this journey?",
                choices: [
                    "A stormy shipwreck where travelers panic in darkness",
                    "Death stopped their \"carriage\" for the persona to give them a trip to the afterlife, showing life from start, 'till the end",
                    "A frantic race against sunset down a mountain highway",
                    "A ghost haunting an abandoned schoolhouse",
                ],
                correct: 1,
                feedback: "Nice reading! School (youth), Grain (maturity), and Setting Sun (old age) trace life into Eternity.",
            },
        ],
    },
    Poem {
        title: "\"Hope\" is the thing with feathers",
        author: "Emily Dickinson (1891)",
        lines: &[
            "\"Hope\" is the thing with feathers -",
            "That perches in the soul -",
            "And sings the tune without the words -",
            "And never stops - at all -",
            "",
            "And sweetest - in the Gale - is heard -",
            "And sore must be the storm -",
            "That could abash the little Bird",
            "That kept so many warm -",
            "",
            "I've heard it in the chillest land -",
            "And on the strangest Sea -",
            "Yet - never - in Extremity,",
            "It asked a crumb - of me.",
        ],
        questions: &[
            PoemQ {
                prompt: "What literary figure governs the entire poem?",
                choices: [
                    "An extended metaphor comparing Hope to a resilient singing bird",
                    "A literal ornithology diary entry",
                    "A cautionary fable about feeding wild birds",
                    "A satirical dialogue with an emperor",
                ],
                correct: 0,
                feedback: "Yes! Hope stays in the spirit through fierce storms and never quits.",
            },
            PoemQ {
                prompt: "What do the closing lines (\"never - in Extremity, / It asked a crumb - of me\") emphasize?",
                choices: [
                    "True hope sustains us selflessly even in our darkest hardship",
                    "The speaker forgot to bring bread on a voyage",
                    "Hope vanishes whenever storms strike the ocean",
                    "Songs are harder to sing in winter weather",
                ],
                correct: 0,
                feedback: "Exactly! Hope asks for nothing in return even in crisis.",
            },
        ],
    },
    Poem {
        title: "Crossing the Bar",
        author: "Alfred, Lord Tennyson (1889)",
        lines: &[
            "Sunset and evening star,",
            "And one clear call for me!",
            "And may there be no moaning of the bar,",
            "When I put out to sea,",
            "",
            "But such a tide as moving seems asleep,",
            "Too full for sound and foam,",
            "When that which drew from out the boundless deep",
            "Turns again home.",
        ],
        questions: &[
            PoemQ {
                prompt: "What does \"put out to sea\" symbolize in this poem?",
                choices: [
                    "A calm, accepting departure at life's twilight",
                    "Opening a busy fishing market before noon",
                    "Fear of violent thunderstorms at harbor",
                    "Searching for lost sunken gold",
                ],
                correct: 0,
                feedback: "Calm twilight imagery turns departure from life into a peaceful voyage home.",
            },
            PoemQ {
                prompt: "What tone is created by \"a tide as moving seems asleep, / Too full for sound and foam\"?",
                choices: [
                    "Serene dignity and quiet reverence",
                    "Bitter regret and panic",
                    "Comedic mockery of sailors",
                    "Frantic wartime urgency",
                ],
                correct: 0,
                feedback: "Clean comprehension! The deep silent tide conveys peaceful acceptance.",
            },
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poems_and_trials_populated() {
        assert_eq!(ALGAE_TRIALS.len(), 3);
        assert!(POEMS.len() >= 3);
        assert_eq!(POEMS[0].questions.len(), 2);
    }
}
