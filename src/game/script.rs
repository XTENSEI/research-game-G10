//! Every line of SCRIPT RES.md, encoded as data. Nothing paraphrased.

/// One spoken line: speaker + text.
pub struct Line {
    pub who: &'static str,
    pub text: &'static str,
}

pub const fn l(who: &'static str, text: &'static str) -> Line {
    Line { who, text }
}

// ============ CANDY LAND OPENING ============

pub const WAKE_SCENE: &[Line] = &[
    l("River", "Where... where am I?"),
    l("River", "This isn't home. This isn't even Earth."),
    l("(Narration)", "River looks around their surroundings and notices a castle."),
    l("(Narration)", "River starts walking on the forest path."),
];

#[allow(dead_code)]
pub const WAKE: Line = l("River", "Where... where am I?");
#[allow(dead_code)]
pub const WAKE2: Line = l("River", "This isn't home. This isn't even Earth.");
#[allow(dead_code)]
pub const LOOK: Line = l("(Narration)", "River looks around their surroundings and notices a castle.");

// ============ JIN ENCOUNTER ============

pub const JIN_INTRO: &[Line] = &[
    l("Jin", "Howdy!"),
    l("Jin", "I was just in the forest earlier grabbing some licorice, could you help me out here? I left my horse at home and whooo boy these sure are a bunch of sticks."),
    l("Jin", "Say, you from around here?"),
];

pub const JIN_WHERE: &[Line] = &[l("Jin", "Well we're in the red licorice forest ofcourse...")];
pub const JIN_NO: &[Line] = &[
    l("Jin", "Oh, well where are you from?"),
    l("Jin", "Aquarius? or Cosma?"),
];

pub const JIN_WARN: &[Line] = &[
    l("Jin", "I gotta warn you about some of the wild life out here.. They're pretty annoying to deal with"),
];

pub const JIN_ANNOY: &[Line] = &[
    l("Jin", "Well, yeah?"),
    l("Jin", "Everything is annoying"),
    l("Jin", "Everyone gives you riddles to solve, questions to answer, just to prove that they're better than you. And there are a lot of them here in the forest."),
    l("Jin", "But I don't bother with that stuff, that's why I live alone, heck it was a risk even asking you for help."),
];

pub const JIN_RISK: &[Line] = &[
    l("Jin", "Yeah it's a risk? I mean you could beat me, that would be bad"),
    l("Jin", "You really aren't from here. Like at all. Let me demonstrate."),
];

pub const JIN_RIDDLE_DEMO: &[Line] = &[
    l("Jin", "Yeah? I mean you could ask me a riddle/question and beat me, that would be bad"),
    l("Jin", "You really aren't from here. Like at all. Let me demonstrate."),
];

pub const JIN_LOSE: &[Line] = &[l("Jin", "Aw shucks, looks like you don't got what it takes.")];
pub const JIN_WIN: &[Line] = &[
    l("Jin", "Wow! You are a natural! Can you help me pick up these licorice sticks now.."),
];

pub const JIN_PERFECT: &[Line] = &[
    l("Jin", "I really do appreciate it, partner! If you want, you could go ahead and take my horse. It's yours, really!"),
];
pub const JIN_LOST_ONE: &[Line] = &[
    l("Jin", "Thanks partner! You better go along then, it's getting pretty late"),
];

pub const JIN_BOOK_OFFER: &[Line] = &[
    l("(Narration)", "River notices a big castle while at Jin's house."),
    l("(Narration)", "They notice a book laying on the table in Jin's house, it's River's family heirloom."),
];

pub const JIN_BOOK_THIS: &[Line] = &[
    l("Jin", "Oh, *that* book? I found it three days earlier. Around the same area I found you!"),
];
pub const JIN_BOOK_MINE: &[Line] = &[
    l("Jin", "Apologies! I actually found it in the same area I found you!"),
];

pub const JIN_BOOK_RETURN: &[Line] = &[
    l("River", "..."),
    l("River", "Could I have it back?"),
    l("Jin", "Ofcourse!"),
];

// ============ GUARDS ============

pub const KANDY_INTRO_SCENE: &[Line] = &[
    l("Kandy Kween", "WHO GOES THERE..?"),
    l("(Narration)", "*Lights turn on*"),
    l("Kandy Kween", "Matt, who is this?"),
    l("Matt", "I am unaware madame."),
    l("Matt", "Whoever it is, they made it through the guards and trials."),
    l("Kandy Kween", "Well that isn't very reassuring, I made those myself."),
    l("(Narration)", "River notices that Kandy Kween's necklace contains an essential ingredient for the elixir."),
];

pub const CLUE_GILL: &[Line] = &[
    l("Gillbert", "hey, do you know where wild sea berries grow?"),
];
pub const CLUE_SCAL: &[Line] = &[
    l("Scalleon", "hmm, I believe it's in the *west*, i think?"),
];
pub const CLUE_LADY: &[Line] = &[
    l("Market Lady", "real ones grow from the *North*. And the city hall? It faces the *east*."),
];

pub const PORTAL_SCENE: &[Line] = &[
    l("(Narration)", "The stone circle blazes to life. The Rainbow Road ignites."),
    l("(Narration)", "But the world itself starts to shake..."),
];

pub const GUARDS: &[Line] = &[
    l("Guard 1", "One of us speaks truth, the other-"),
    l("Guard 2", "you don't seem like you're from around here"),
    l("Guard 2", "Oh we're sticking to the script? My bad."),
    l("Guard 2", "Yes, whatever he said."),
    l("Guard 1", "You know what, I'm not even in the mood for it anymore"),
    l("Guard 2", "No but we were gonna do it"),
    l("Guard 2", "I mean we have to do it."),
    l("Guard 1", "Yeah well you ruined it"),
    l("(Narration)", "The guards fight, letting River pass through quietly."),
];

// ============ KANDY KWEEN ============

#[allow(dead_code)]
pub const KANDY_INTRO: &[Line] = &[
    l("Kandy Kween", "WHO GOES THERE..?"),
    l("(Narration)", "*Lights turn on*"),
    l("Kandy Kween", "Matt, who is this?"),
    l("Matt", "I am unaware madame."),
    l("Matt", "Whoever it is, they made it through the guards and trials."),
    l("Kandy Kween", "Well that isn't very reassuring, I made those myself."),
    l("(Narration)", "River notices that Kandy Kween's necklace contains an essential ingredient for the elixir."),
];

pub const KANDY_GREAT_TEXTS: &[Line] = &[
    l("Kandy Kween", "Thanks, here's your reward..."),
];
pub const KANDY_EASY_TEXTS: &[Line] = &[
    l("(Narration)", "Kandy Kween starts fuming"),
    l("Kandy Kween", "OH REALLY ??"),
];
pub const KANDY_SUCKED_TEXTS: &[Line] = &[
    l("(Narration)", "Kandy Kween starts fuming"),
    l("Kandy Kween", "NOW WERE THEY ?"),
    l("Kandy Kween", "YOU KNOW WHAT ELSE SUCKS ?"),
    l("River", "..."),
    l("River", "no"),
];

pub const TRAPDOOR: &[Line] = &[
    l("(Narration)", "Kandy Kween activates a trap door regardless of which dialogue option you pick."),
    l("(Narration)", "River is dropped in this candy cave that transitions into an actual rock cave."),
    l("(Narration)", "They plummet into the water and notice that what surrounds them are remains of candy people."),
];

// ============ QTE NARRATION ============

pub const QTE_LINES: &[Line] = &[
    l("(Narration)", "River dives down into the water."),
    l("(Narration)", "River sees a light down in the depths."),
    l("(Narration)", "The light becomes bigger, River noticed this and tries to swim back up."),
];

pub const DRAGGED: &[Line] = &[
    l("(Narration)", "Something drags River down into the depths."),
    l("(Narration)", "*pitch black*"),
];

// ============ INTRO (house) ============

/// Marker scene: the intro renders as picture panels, not dialogue.
pub const INTRO_SCENE: &[Line] = &[l("(Intro)", "intro")];

pub const INTRO_LINES: &[&str] = &[
    "Nadia has fallen sick.",
    "The doctors call it \"illiteracy\".",
    "The Grandparent gives River the Family Heirloom.",
    "A book that might be the key.",
    "River thinks nothing of it.",
    "Until they try to read it...",
];

// ============ RIDDLES (one-try only) ============

pub struct Riddle {
    pub question: &'static str,
    pub options: &'static [&'static str],
    pub answer: usize,
    #[allow(dead_code)]
    pub biome: Biome,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Biome { Candy, Aquarius, Cosma, Home }

pub const CANDY_RIDDLES: &[Riddle] = &[
    Riddle {
        question: "\"He always obfuscates his instructions for the candy housing projects because it's obvious that he wants us to fail and be humiliated.\" \"I can't even understand your plan, it's like you're obfuscating it.\"\nBased on the statements above, what does OBFUSCATE mean?",
        options: &[
            "Harshness in tone or behavior due to personal reasons",
            "Faking or fabricating of evidence",
            "Making something vague and unintelligible",
            "A confused mixture",
        ],
        answer: 2,
        biome: Biome::Candy,
    },
    Riddle {
        question: "What word was correctly spelled?",
        options: &["Acommodate", "Acomodate", "Accomodate", "Accommodate"],
        answer: 3,
        biome: Biome::Candy,
    },
];

pub const COSMA_RIDDLES: &[Riddle] = &[
    Riddle {
        question: "The more of me you discover, the less you actually know. I outweigh everything you can see, yet you cannot see me at all.",
        options: &[
            "Knowledge",
            "Darkness",
            "The ocean",
            "Time",
        ],
        answer: 0,
        biome: Biome::Cosma,
    },
    Riddle {
        question: "Short story: Mina saved her coins for a year. On market day she walked past the toy stall, past the sweet stall, and stopped at the book cart. She traded every coin for one worn volume, hugged it to her chest, and walked home barefoot and smiling.\nWhat does the story suggest about Mina?",
        options: &[
            "She values learning above material things",
            "She lost her shoes at the market",
            "She could not afford toys or sweets",
            "She dislikes going to the market",
        ],
        answer: 0,
        biome: Biome::Cosma,
    },
];

pub const AQUARIUS_RIDDLES: &[Riddle] = &[
    Riddle {
        question: "Which italicization correctly specifies that Gillbert specifically does not particularly believe in Neptune?",
        options: &[
            "Scalleon said that Gillbert *doesn't* believe in Neptune",
            "Scalleon said that *Gillbert* doesn't believe in Neptune",
            "Scalleon *said* that Gillbert doesn't believe in Neptune",
            "Scalleon said that Gillbert doesn't believe in *Neptune*",
        ],
        answer: 3,
        biome: Biome::Aquarius,
    },
    Riddle {
        question: "Based on this dimension, what is Neptune?",
        options: &["Their god", "Their supreme leader", "A book", "A region they live in"],
        answer: 0,
        biome: Biome::Aquarius,
    },
];
