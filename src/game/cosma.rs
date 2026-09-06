//! The Rainbow Road. Catch stars, fill meter, carve slabs, light torches,
//! summon the genie, answer the meaning of the poem. New poem each retry.

pub struct Poem {
    pub lines: &'static [&'static str],
    pub question: &'static str,
    pub options: &'static [&'static str],
    pub answer: usize,
}

pub const POEMS: &[Poem] = &[
    Poem {
        lines: &[
            "I am the word not yet spoken,",
            "the road not yet taken,",
            "the seed inside the stone.",
            "Feed me letters and I grow;",
            "starve me and I stay alone.",
        ],
        question: "What is the meaning of the whole poem?",
        options: &[
            "Literacy is a seed: unused words wither, but learning makes them grow",
            "Words are dangerous and should stay buried",
            "Stones can grow into trees if you water them",
            "Speech is meaningless without an audience",
        ],
        answer: 0,
    },
    Poem {
        lines: &[
            "Two eyes, one tongue, blind until read,",
            "I sleep on shelves and wake in heads.",
            "Some call me wisdom, some call me art,",
            "I carry whole worlds in a single heart.",
        ],
        question: "What is the meaning of the whole poem?",
        options: &[
            "A book: dormant knowledge that awakens through reading",
            "A snake that lives in libraries",
            "A sleeping giant who guards treasure",
            "The moon, which rises when opened",
        ],
        answer: 0,
    },
    Poem {
        lines: &[
            "The more you take from me, the more you leave behind,",
            "yet I am not diminished, nor are you confined.",
            "I am the trail of every step, the story of the walk,",
            "the proof you moved, the proof you learned, the silence after talk.",
        ],
        question: "What is the meaning of the whole poem?",
        options: &[
            "Footsteps: every step taken leaves a mark without loss",
            "Water evaporating from a bucket",
            "Money spent on books",
            "A shadow that follows you",
        ],
        answer: 0,
    },
];

/// Star catching config.
pub const STAR_TARGET: u32 = 20;      // meter full at N stars
pub const DYING_PENALTY: u32 = 3;     // dying star removes this many
pub const LIVES: u32 = 3;             // caught dying star costs 1 life
pub const SLABS: u32 = 3;             // slabs = torches
