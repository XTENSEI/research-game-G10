//! Jin tutorial battle: comprehension + basic English questions.
//! Ported from riddles/jin-battle-questions.json.

pub struct JinQ {
    pub question: &'static str,
    pub options: &'static [&'static str],
    pub correct: usize,
}

pub const FULL_SET: &[JinQ] = &[
    JinQ {
        question: "Where did Jin say he was earlier?",
        options: &["In the candy castle", "In the red licorice forest", "At the market", "By the river"],
        correct: 1,
    },
    JinQ {
        question: "What was Jin looking for in the forest?",
        options: &["Gold coins", "Licorice sticks", "His horse", "A way home"],
        correct: 1,
    },
    JinQ {
        question: "What does Jin say everyone does in the forest?",
        options: &["They give you riddles and questions", "They sell candy", "They fight you", "They run away"],
        correct: 0,
    },
    JinQ {
        question: "Why does Jin live alone?",
        options: &["He likes quiet", "He avoids riddles and questions", "He has no friends", "He is hiding"],
        correct: 1,
    },
    JinQ {
        question: "What does 'obfuscate' mean?",
        options: &["To celebrate", "To make vague and unintelligible", "To break something", "To run fast"],
        correct: 1,
    },
    JinQ {
        question: "Which is the correct spelling?",
        options: &["Acommodate", "Accomodate", "Accommodate", "Acomadate"],
        correct: 2,
    },
    JinQ {
        question: "What did Jin say was a 'risk'?",
        options: &["Asking River for help", "Going to the castle", "Eating licorice", "Sleeping alone"],
        correct: 0,
    },
];

/// Used when River lost at least once and retries.
pub const REDUCED_SET: &[JinQ] = &[
    JinQ {
        question: "Where was Jin earlier?",
        options: &["The castle", "The red licorice forest", "The ocean", "The mountain"],
        correct: 1,
    },
    JinQ {
        question: "What was Jin looking for?",
        options: &["Licorice sticks", "His sword", "A map", "Food"],
        correct: 0,
    },
    JinQ {
        question: "Why does Jin live alone?",
        options: &["He is rich", "He avoids riddles and questions", "He is a king", "He lost his family"],
        correct: 1,
    },
];
