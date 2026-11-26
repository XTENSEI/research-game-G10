//! Full dialogue, riddles, and biome progression matching SCRIPT RES.pdf.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Biome {
    Kelp,
    Coral,
    Slums,
    Home,
}

#[derive(Clone, Copy)]
pub struct Line {
    pub who: &'static str,
    pub expr: &'static str,
    pub text: &'static str,
}

impl Line {
    pub const fn new(who: &'static str, text: &'static str) -> Self {
        Self { who, expr: "normal", text }
    }
    pub const fn expr(who: &'static str, expr: &'static str, text: &'static str) -> Self {
        Self { who, expr, text }
    }
}

#[derive(Clone, Copy)]
pub struct Riddle {
    pub biome: Biome,
    pub title: &'static str,
    pub prompt: &'static str,
    pub choices: [&'static str; 4],
    pub correct: usize,
    pub explain: &'static str,
}

pub fn biome_title(b: Biome) -> &'static str {
    match b {
        Biome::Kelp => "BIOME 1: KELP FOREST & CRAB CASTLE",
        Biome::Coral => "BIOME 2: CORAL CITY (AQUARIUS)",
        Biome::Slums => "BIOME 3: SOMBER SLUMS & RAINBOW ROAD",
        Biome::Home => "HEARTHSIDE COTTAGE",
    }
}

// ---------------------------------------------------------------------------
// Intro Storybook (4 Undertale plates: assets/game/intro/plate_0..3.png)
// ---------------------------------------------------------------------------

pub const INTRO_PLATES: &[(usize, &str, &str)] = &[
    (
        0,
        "Nadia",
        "Little Nadia fell gravely ill with *illiteracy*. Every letter drifts away in a blur whenever she opens a page.",
    ),
    (
        1,
        "Mark",
        "Grandparent and elder sibling Mark taught day and night. River searched through books, archives, and tales for a cure.",
    ),
    (
        2,
        "River",
        "Worn out from searching, River rested on the moonlit beach until a warm, *glowing* *storybook* washed onto the sand.",
    ),
    (
        3,
        "River",
        "As River dozed beside the open book, rings of light pulled them down beneath the tide into another realm...",
    ),
];

// ---------------------------------------------------------------------------
// Biome 1: Kelp Forest & Crab Castle (pp. 1-3, 15-16)
// ---------------------------------------------------------------------------

pub const KELP_ARRIVE: &[Line] = &[
    Line::expr("River", "shocked", "Huh...? Wait, why is there drag around my arms?!"),
    Line::expr("River", "shocked", "What is all this tall green stuff swaying everywhere... kelp?!"),
    Line::new("River", "Am I... *in* *water*?! Hang on, I can breathe down here!"),
    Line::expr("River", "shocked", "WHOA! What is THAT?! A giant red crab with a *kelp* *slingbag*?!"),
];

pub const KENNY_QTE_PASS: &[Line] = &[
    Line::new("River", "Phew... *that* *was* *close...* Looks like I outswam whatever that thing was."),
    Line::expr("Kenny", "worried", "*panting* Hey! Wait up! Stop swimming like a torpedo!"),
    Line::expr("River", "shocked", "Gah!! Wait, didn't I leave you back at the kelp bend?! What do you want?!"),
    Line::expr("Kenny", "worried", "Look friend, I'm just tryna help you! You looked like you were having a panic attack."),
];

pub const KENNY_QTE_FAIL: &[Line] = &[
    Line::new("River", "Aw *shucks*, my boots got tangled in the sea-grass!!"),
    Line::expr("Kenny", "happy", "Gotcha! Tap, tap. Relax, kid! I wasn't chasing you to pinch you!"),
    Line::expr("Kenny", "worried", "You looked like you were having a panic attack. I'm just tryna help you!"),
];

pub const KENNY_INTRO: &[Line] = &[
    Line::expr("Kenny", "normal", "You ready to chat cleanly now, surface-kid?"),
];

pub const KENNY_HOME_Q: &str = "Coral City, I presume?";

pub const KENNY_WORLD_EXPLAIN: &[Line] = &[
    Line::expr("Kenny", "normal", "Well regardless... I gotta warn you about some of the people out here before you wander off!"),
    Line::expr("Kenny", "normal", "Everyone hits you with riddles and questions just to prove something. *The* *King* *prohibited* *all* *weapons* and physical brawling a while back!"),
    Line::expr("Kenny", "worried", "Honestly, coming over here was a *risk*! You could've been tricking me!"),
];

pub const KENNY_PRE_DUEL: &[Line] = &[
    Line::expr("Kenny", "normal", "You really aren't from here. Like at all. Let me demonstrate!"),
];

pub const KENNY_RETRY_PROMPT: &str = "Aw shucks, looks like you don't got what it takes yet. Want to try again with fewer questions?";

pub const KENNY_FLAWLESS_WIN: &[Line] = &[
    Line::expr("Kenny", "happy", "Whoa! You are a natural! Not a single scratch AND every question nailed!"),
    Line::expr("Kenny", "happy", "I don't carry spare shells on me, so welcome to my humble abode by the grove! Here, take this *Swift* *Current* charm and keep that old storybook close!"),
    Line::new("River", "Hey, that ledger outside has the exact crest from the beach! I can dash right through kelp currents now!"),
];

pub const KENNY_EVEN_WIN: &[Line] = &[
    Line::expr("Kenny", "normal", "Okay! Now that we're even, you'd better head along toward the plains. It's getting pretty late."),
    Line::new("River", "Look on that stone ledger! The glowing book! Time to find the Queen's elixir."),
];

pub const GUARDS_SCENE: &[Line] = &[
    Line::new("Guard 1", "Halt! One of us only speaks truth, and the other..."),
    Line::new("Guard 2", "You look weird. Are you from around here?"),
    Line::new("Guard 2", "Oh, we're sticking to the decree script? My bad! Yes, whatever he said."),
    Line::new("Guard 1", "You know what, I'm not even in the mood anymore, Carl. You ruined it!"),
    Line::new("Guard 2", "No, we were literally supposed to do the knight riddle!!"),
    Line::new("River", "...Did they really just wander off arguing over their own lines? I'll slip right through!"),
];

pub const QUEEN_CRAB_INTRO: &[Line] = &[
    Line::expr("Queen Crab", "normal", "WHO GOES THERE..? Lights!!"),
    Line::new("Sophie", "*Honk-jingle!* Hold on, Madame, flipping the candelabra switch now!"),
    Line::expr("Queen Crab", "normal", "Sophie, *who* *is* *this*? Some dry-land surface dweller?!"),
    Line::new("Sophie", "I am unaware, Madame! Whoever it is, they made it past the guards and trials... looking kinda like a *mutated* *sea* *pickle*."),
    Line::expr("River", "shocked", "Hey!! I'm River! And that glowing pearl around your neck... that's an *Elixir* ingredient, isn't it?!"),
];

pub const QUEEN_EVAL_Q: &str = "Well that isn't very reassuring. Wait, what did you think of my guards and puzzles?";

pub const QUEEN_GREAT_REPLY: &[Line] = &[
    Line::expr("Queen Crab", "normal", "Why, thank you! Here is your royal reward..."),
    Line::expr("Queen Crab", "normal", "A ONE-WAY TICKET DOWN THE *TRAPDOOR*!! SEIZE THE ELIXIR NEVERMORE!"),
];

pub const QUEEN_EASY_REPLY: &[Line] = &[
    Line::expr("Queen Crab", "fuming", "OH REALLY?! I stayed up writing those *LONE* *STONE* clauses MYSELF!!"),
    Line::expr("Queen Crab", "fuming", "ENJOY THE *TRAPDOOR*!!"),
];

pub const QUEEN_SUCK_REPLY: &[Line] = &[
    Line::expr("Queen Crab", "fuming", "NOW WERE THEY?! YOU KNOW WHAT *ELSE* SUCKS?!"),
    Line::expr("River", "shocked", "...No??"),
    Line::expr("Queen Crab", "fuming", "THE *TRAPDOOR* UNDER YOUR BOOTS!!"),
];

pub const QUEEN_DROP: &[Line] = &[
    Line::expr("River", "shocked", "WAAAAH!! Wait, I grabbed the glowing *Pearl* *of* *Clarity* on the way down!!"),
];

// ---------------------------------------------------------------------------
// Biome 2: Coral City / Aquarius (pp. 3-9, 12-13, 16-17, 20)
// ---------------------------------------------------------------------------

pub const AQUA_WAKE: &[Line] = &[
    Line::new("River", "[...Pitch black. Dim aquarium lanterns slowly drift into focus...]"),
    Line::expr("Jack", "normal", "Hey buddy! Easy now, are you good? Fascinating, you fell three hundred fathoms and didn't crumble!"),
    Line::expr("Jill", "shocked", "Hi!! Sorry for tying you up, we weren't sure if you were alive or not!"),
    Line::expr("River", "shocked", "Ow... Where am I?! Two swordfish in lab suits?! Hey, can you guys untie me?!"),
    Line::expr("Jill", "normal", "Woops! Right, why did we tie you up anyway? Jack?!"),
    Line::expr("Jack", "normal", "What do you mean 'We'? Are you speaking French? YOU tied them up!"),
    Line::expr("Jill", "normal", "Well of all the *weird* *looking* *fish*... I'm Jill and this is Jack. We're siblings, can you tell?"),
    Line::expr("River", "normal", "Wow, I wouldn't have known if you hadn't told me."),
    Line::expr("Jill", "shocked", "Wait a minute, are you messing with me?! Doing that... scar- thing? *Sarcanistic*?!"),
    Line::expr("Jack", "normal", "It's called *sarcasm*, Jill. And stop correcting me! Look, our apothecary is short three *rare* *reef* *ingredients*."),
    Line::expr("Jill", "normal", "Bring us a *Blue* *Seashroom*, a *Tentacle* *Anemone*, and a real *True* *Seaberry* from the market, and we'll get you a ride!"),
    Line::expr("Jack", "normal", "Be careful with the berries: listen to Market clues first, then take our *shore* *cut* (pun intended) to Grandpa Tow-Tou's garage!"),
];

pub const ALLEY_DUEL: &[Line] = &[
    Line::new("AngelFish", "If thy fields are as parched as thy coin purse, then state it!!"),
    Line::new("StoneFish", "Thy argument is as penetrable as thy armor, fancy fins! Remember the King banned fists, only sharpest *words*!"),
    Line::new("River", "Whoa... even alley arguments down here are medieval vocabulary duels."),
];

pub const CLUE_GILL_SCAL: &[Line] = &[
    Line::new("Gillbert", "Looking for the *True* *Seaberry* for Jack & Jill? Careful! Fake kelp-berries are everywhere!"),
    Line::new("Scalleon", "Harbormaster log says check the *WEST* market terrace. Don't follow bad rumors!"),
    Line::new("Gillbert", "Wait, don't stop there! Ask the *Old* *Market* *Lady* and check *Neptune's* *Statue* at City Hall too!"),
];

pub const MARKET_LADY_ASK: &str = "Sweetheart, hunting for True Seaberries? Tell an old reef granny, did Scalleon send you nicely?";

pub const MARKET_LADY_CLUE: &[Line] = &[
    Line::new("Market Lady", "Bless your manners! *NORTH* is where I've seen and eaten those sweet *True* *Seaberries*! Check the northern market terrace!"),
];

pub const NEPTUNE_STATUE_EXAMINE: &[Line] = &[
    Line::new("River", "An old moss-carved *Statue* *of* *Neptune* towering beside City Hall!"),
    Line::new("River", "Look at his trident and the carved compass base: Neptune points *strictly* *EAST*!"),
];

pub const TRUE_SEABERRY_FOUND: &[Line] = &[
    Line::expr("River", "normal", "Putting together Gillbert's *West*, Market Lady's *North*, and Neptune's *East* clue..."),
    Line::expr("River", "normal", "Look under this glowing bush! Deep blue berries: a real *True* *Seaberry*!!"),
];

pub const DECOY_BERRY_WARN: &[Line] = &[
    Line::expr("River", "shocked", "Ew, sour ink-berry! That one's fake. Let me check `[Tab] Field Notes` for all three NPC clues first!"),
];

pub const SEASHROOM_PICK: &[Line] = &[
    Line::new("River", "Found a pulsing *Blue* *Seashroom* growing beside the reef arch! One glowing fungus down."),
];

pub const ANEMONE_PICK: &[Line] = &[
    Line::new("River", "Snipped a *Tentacle* *Anemone* frond! That's another elixir ingredient in the bag."),
];

pub const TOWTOU_GREET: &[Line] = &[
    Line::new("Jill", "Graaamps!! Hey Graaaammmppsss!!"),
    Line::new("Tow-Tou", "WHAAAAT?! I'm tryna sleep here! First you wander into my garage, now you demand rides?!"),
    Line::new("Jack", "All three Aquarius reagents are packed neat! Plus Queen Crab insulted River for looking like a *mutated* *sea* *pickle*."),
    Line::expr("Tow-Tou", "normal", "WHAT?! Okay, she's not wrong... STILL!! I won't stand for it! Hop in, kid! We shall free you from this ocean of stone!!"),
];

pub const ANGLERS_AMBUSH: &[Line] = &[
    Line::new("Angler A", "*Click-flash!* Hold it, shell-taxi! Nobody crosses Jevil Ridge without surviving our cypher!"),
    Line::new("Angler B", "The King banned fists, yeah, so our *Three* *Sea* *Anglers* settle tolls on the *MIC*!"),
    Line::new("Angler C", "Drop clean bars on beat and pick the sharpest rhyme, kid, or walk home backwards!!"),
];

pub const ANGLERS_WIN: &[Line] = &[
    Line::new("Angler A", "*Wait...* that internal rhyme scheme was actually fire?!"),
    Line::new("Tow-Tou", "Ha!! Never doubt a kid carrying a glowing storybook! Ahead sits the *Grave* *Cave* *Maze* *of* *Craze*. Watch out, wrong turns push you back without a sound!"),
];

// ---------------------------------------------------------------------------
// Biome 3: Somber Slums & Rainbow Road (pp. 9, 14, 17-21)
// ---------------------------------------------------------------------------

pub const SLUMS_ARRIVE: &[Line] = &[
    Line::new("River", "Brrr... the warm lagoon vanished. Just sunken shipwrecks, whale bones, and dark abyssal shacks down here..."),
    Line::new("Thaumiel", "Hold your lantern high, traveler. The *Somber* *Slums* swallow anyone who rushes blindly."),
    Line::new("River", "Oh! You're an anglerfish... with sky-blue lantern light! Who's blocking the bone bridge?"),
    Line::new("Tenebris", "*...Hmph.* Name's *Tenebris*. Only folks who read beneath the shadows cross my bridge. Ready to test your wits?"),
];

pub const ALGAE_TEMPLE_PRE: &[Line] = &[
    Line::new("Thaumiel", "Here stands the forgotten *Temple* *of* *The* *Figurate* *Algae*, where poetry and number patterns fuse into alchemy."),
    Line::new("Thaumiel", "Solve all three figure riddles to blend your Aquarius ingredients with Queen Crab's pearl!"),
];

pub const ALGAE_TEMPLE_POST: &[Line] = &[
    Line::new("Thaumiel", "Look at that bright blue glow! The *Elixir* *of* *Literacy* is complete inside its crystal vial!"),
    Line::new("River", "Only *The* *Rainbow* *Road* left between us and the portal home... Wait, why is the sky cracking behind us?!"),
];

pub const RAINBOW_GENIE_INTRO: &[Line] = &[
    Line::new("Magic Fire Genie", "HALT, SEEKER OF LETTERS! Light the Rainbow Road and prove you truly *understand* verse!"),
    Line::new("River", "Try me! Nadia's waiting at home!"),
];

pub const CRUMBLE_START: &[Line] = &[
    Line::expr("River", "shocked", "WHAT?! THE FLOOR IS SHATTERING!!"),
    Line::expr("Magic Fire Genie", "normal", "THIS REALM WAS HELD TOGETHER BY ANCIENT WORDS! RUN THROUGH THE PORTAL NOW!!"),
];

pub const HOME_ENDING: &[Line] = &[
    Line::expr("River", "normal", "*Gasp...!* Warm morning light... wood floors... I made it back before the last shard broke!!"),
    Line::new("Mark", "River!! Where were you all night?! We checked the beach! Wait, what's glowing in your bag?!"),
    Line::new("Grandparent", "By the stars... Is that the *Elixir* from the old chronicle?! Quick, give a spoonful to Nadia!"),
    Line::expr("Nadia", "cured", "...\"*Once* *upon* *a* *moonlit* *tide...*\" Wait! The words aren't blurry anymore!! I can read EVERY letter!!"),
    Line::expr("River", "normal", "You're cured, Nadia!! Every riddle, rhyme, and puzzle brought every word back home."),
];

// ---------------------------------------------------------------------------
// Locked 1-Try Riddles (2-3 per biome, verbatim SCRIPT RES.pdf pp. 15-19)
// ---------------------------------------------------------------------------

pub const RIDDLES: &[Riddle] = &[
    // --- Kelp Forest & Crab Castle ---
    Riddle {
        biome: Biome::Kelp,
        title: "Kelp Monolith I: Vocabulary",
        prompt: "\"Due to the complexity of the sandstone highway projects, critics say the city has used unclear terminology to OBFUSCATE the project's real environmental cost.\"",
        choices: [
            "a) Making something clearer and transparent",
            "b) Highlighting important project details",
            "c) Making something vague and unintelligible",
            "d) Summarizing findings concisely",
        ],
        correct: 2,
        explain: "'Obfuscate' means deliberately making meaning unclear, obscure, or confusing.",
    },
    Riddle {
        biome: Biome::Kelp,
        title: "Kelp Monolith II: Orthography",
        prompt: "Only one tablet preserves the royal spelling decree. Which is spelled correctly?",
        choices: [
            "a) Acommodate",
            "b) Acomodate",
            "c) Accomodate",
            "d) Accommodate",
        ],
        correct: 3,
        explain: "'Accommodate' takes BOTH double 'c' and double 'm', with plenty of room for two pairs!",
    },
    Riddle {
        biome: Biome::Kelp,
        title: "Kelp Monolith III: Clause Repair",
        prompt: "Which revision fixes this comma splice cleanly?\n\"The lagoon glimmered, we followed the kelp path.\"",
        choices: [
            "a) \"The lagoon glimmered, so we followed the kelp path.\"",
            "b) \"The lagoon glimmered, because we followed.\"",
            "c) \"The lagoon glimmered, yet however we followed.\"",
            "d) Leave the comma splice unchanged",
        ],
        correct: 0,
        explain: "Two independent clauses need a coordinating conjunction (, so / , and), semicolon, or period.",
    },
    // --- Coral City / Aquarius ---
    Riddle {
        biome: Biome::Coral,
        title: "Coral Tablet I: Italic Emphasis",
        prompt: "What does italicizing *Neptune* imply in:\n\"Scalleon said that Gillbert doesn't believe in *Neptune*\"?",
        choices: [
            "a) Gillbert doubts Scalleon himself",
            "b) Gillbert only whispers the name",
            "c) Nobody lives in Aquarius anymore",
            "d) Gillbert believes in other gods, just not *Neptune* specifically",
        ],
        correct: 3,
        explain: "Contrastive italics single out *Neptune* as the specific exception among deities.",
    },
    Riddle {
        biome: Biome::Coral,
        title: "Coral Tablet II: Reading in Context",
        prompt: "Based on the statues, proverbs, and dialogue across this sunken dimension, who/what is *Neptune*?",
        choices: [
            "a) Their god",
            "b) A rival crab merchant",
            "c) A lost library book",
            "d) The bridge toll collector",
        ],
        correct: 0,
        explain: "Belief debates at City Hall and sacred reef monuments establish *Neptune* as their deity.",
    },
    Riddle {
        biome: Biome::Coral,
        title: "Coral Tablet III: Precision Word Choice",
        prompt: "Which word correctly completes AngelFish's duel rebuttal?\n\"Nothing you shout can ___ our city's grammar standards!\"",
        choices: [
            "a) effect",
            "b) affect",
            "c) infect",
            "d) defect",
        ],
        correct: 1,
        explain: "'Affect' is the verb meaning to influence or alter; 'effect' is normally the resulting noun.",
    },
    // --- Somber Slums ---
    Riddle {
        biome: Biome::Slums,
        title: "Abyss Bridge I: Metaphor",
        prompt: "Tenebris's First Whisper:\n\"The more of me you discover, the larger I become. What am I?\"",
        choices: [
            "a) The unknown",
            "b) A burning candle",
            "c) Sunken treasure",
            "d) An hourglass",
        ],
        correct: 0,
        explain: "As you learn more, the boundary touching what remains unexplained grows wider: *the unknown*.",
    },
    Riddle {
        biome: Biome::Slums,
        title: "Abyss Bridge II: Elegy Imagery",
        prompt: "Tenebris's Second Verse:\n\"Once you were here, now a memory in my hand.\nI stand by the water, ready to leave you as planned.\nI open my fingers and let the current take you free.\nYour fire burns bright, yet now part of the vast blue sea.\"",
        choices: [
            "a) Letting a pet fish swim away",
            "b) Dropping glass lanterns overboard",
            "c) Scattering a loved one's ashes",
            "d) Torn pages lost to sea foam",
        ],
        correct: 2,
        explain: "Held in hand by the water and releasing ashes into the sea describes scattering a loved one's ashes.",
    },
];

#[cfg(test)]
pub fn biome_riddles(b: Biome) -> Vec<(usize, &'static Riddle)> {
    RIDDLES
        .iter()
        .enumerate()
        .filter(|(_, r)| r.biome == b)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_biome_has_two_to_three_riddles() {
        for b in [Biome::Kelp, Biome::Coral, Biome::Slums] {
            let n = biome_riddles(b).len();
            assert!((2..=3).contains(&n), "{:?} had {} riddles", b, n);
        }
    }
}
