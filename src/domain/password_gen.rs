use aes_gcm::aead::OsRng;
use anyhow::Result;
use rand::RngCore;

const SYMBOL_SET: &str = "!@#$%^&*()-_=+[]{}|;:,.<>?";

// EFF Short Wordlist (1296 words, 4-6 characters each)
const WORD_LIST: &[&str] = &[
    "able", "acid", "acre", "acts", "aged", "aids", "aims", "also", "amid", "anna", "anne", "ants",
    "arch", "area", "args", "arms", "army", "arts", "asia", "aunt", "auto", "away", "baby", "back",
    "ball", "band", "bank", "barn", "base", "bath", "beam", "bean", "bear", "beat", "been", "beer",
    "bell", "belt", "bend", "beta", "bike", "bill", "bind", "bird", "blow", "blue", "boat", "body",
    "bold", "bolt", "bomb", "bond", "bone", "book", "boom", "boot", "bore", "born", "boss", "both",
    "bowl", "bulk", "bull", "burn", "bush", "busy", "café", "cage", "cake", "calf", "call", "calm",
    "came", "camp", "cane", "cape", "card", "care", "carl", "carr", "cars", "cart", "case", "cash",
    "cast", "cats", "cave", "cell", "chad", "chap", "chef", "chen", "chip", "cite", "city", "clan",
    "clay", "clip", "club", "clue", "coal", "coat", "code", "coin", "cold", "cole", "coli", "come",
    "cone", "cook", "cool", "cope", "copy", "cord", "core", "cork", "corn", "cost", "coup", "cows",
    "cozy", "crab", "crew", "crop", "cruz", "cuba", "cube", "cult", "cure", "cute", "dale", "dame",
    "damn", "dana", "dane", "dare", "dark", "data", "date", "dave", "dawn", "days", "dead", "deaf",
    "deal", "dean", "dear", "debt", "deck", "deed", "deer", "dell", "demo", "deny", "desc", "desk",
    "dial", "dice", "dick", "died", "dies", "diet", "dime", "dirt", "disc", "dish", "disk", "dive",
    "dock", "docs", "does", "dogs", "doll", "dome", "done", "doom", "door", "dose", "doug", "dove",
    "down", "drag", "draw", "drew", "drop", "drug", "drum", "dual", "duck", "duct", "duke", "dull",
    "dump", "dune", "dusk", "dust", "duty", "each", "earl", "earn", "ears", "ease", "east", "easy",
    "eats", "edge", "edit", "eyed", "eggs", "epic", "erie", "euro", "even", "ever", "evil", "exam",
    "exit", "face", "fact", "fade", "fail", "fair", "fall", "fame", "fang", "fare", "farm", "fast",
    "fate", "fax", "fear", "feat", "feed", "feel", "fees", "feet", "fell", "felt", "fest", "file",
    "fill", "film", "find", "fine", "fire", "firm", "fish", "fist", "five", "flag", "flat", "flaw",
    "fled", "flee", "flew", "flex", "flip", "flow", "flux", "foam", "foil", "fold", "folk", "fond",
    "font", "food", "fool", "foot", "ford", "fork", "form", "fort", "foul", "four", "fred", "free",
    "fuel", "full", "fund", "fury", "fuse", "fuss", "gain", "gale", "game", "gang", "gaps", "gary",
    "gate", "gave", "gear", "gene", "gift", "gig", "gill", "girl", "give", "glad", "glen", "glow",
    "glue", "goal", "goat", "goes", "gold", "golf", "gone", "good", "gore", "gown", "grab", "grad",
    "graf", "gram", "gray", "greg", "grew", "grey", "grid", "grim", "grin", "grip", "grow", "grub",
    "gulf", "gull", "guns", "guru", "guys", "hail", "hair", "half", "hall", "halt", "hand", "hang",
    "hans", "hard", "harm", "hart", "hash", "hate", "have", "hawk", "haze", "head", "heal", "heap",
    "hear", "heat", "heel", "held", "hell", "helm", "help", "hemp", "herb", "here", "hero", "hide",
    "high", "hike", "hill", "hint", "hire", "hits", "hive", "hold", "hole", "holy", "home", "hong",
    "hood", "hook", "hope", "horn", "hose", "host", "hour", "huge", "hugo", "hull", "hung", "hunt",
    "hurt", "hyde", "hymn", "icon", "idea", "idle", "idol", "inch", "info", "init", "ions", "iowa",
    "iran", "iraq", "iris", "iron", "isle", "item", "ivan", "jack", "jade", "jail", "jake", "jane",
    "jazz", "jean", "jeff", "jess", "jets", "jews", "jill", "joan", "jobs", "joel", "joey", "john",
    "join", "joke", "jose", "juan", "judy", "july", "jump", "june", "junk", "jury", "just", "kate",
    "keen", "keep", "kent", "kept", "kern", "keys", "kick", "kids", "kill", "kind", "king", "kirk",
    "kiss", "kits", "knee", "knew", "knit", "knot", "know", "knox", "kong", "kurt", "lace", "lack",
    "lady", "laid", "lain", "lair", "lake", "lamb", "lame", "lamp", "land", "lane", "lang", "laps",
    "lars", "lash", "last", "late", "lava", "lawn", "laws", "lazy", "lead", "leaf", "leak", "lean",
    "leap", "left", "legs", "lena", "lend", "lens", "lent", "leon", "less", "levy", "liar", "lice",
    "lick", "lied", "lies", "life", "lift", "like", "lily", "lima", "limb", "lime", "line", "ling",
    "link", "lion", "lips", "lisa", "lisp", "list", "live", "load", "loaf", "loan", "lock", "loft",
    "logo", "logs", "lois", "lone", "long", "look", "loop", "lord", "lore", "lose", "loss", "lost",
    "lots", "loud", "love", "luck", "lucy", "luke", "lung", "lust", "lynn", "lyon", "mach", "made",
    "mail", "main", "make", "male", "mali", "mall", "malt", "mann", "many", "maps", "marc", "mare",
    "mark", "mars", "mart", "mary", "mash", "mask", "mass", "mast", "mate", "math", "matt", "mayo",
    "maze", "mead", "meal", "mean", "meat", "meek", "meet", "mega", "melt", "memo", "menu", "mere",
    "mesa", "mesh", "mess", "meta", "mice", "midi", "mild", "mile", "milk", "mill", "milo", "mime",
    "mind", "mine", "mini", "mint", "miss", "mist", "mite", "mode", "mold", "mole", "monk", "mono",
    "mood", "moon", "moor", "more", "moss", "most", "moth", "move", "much", "mule", "must", "mute",
    "myth", "nail", "name", "nano", "nasa", "nash", "navy", "nazi", "neal", "near", "neat", "neck",
    "need", "neil", "neon", "nest", "nets", "news", "next", "nice", "nick", "nina", "nine", "node",
    "none", "noon", "norm", "nose", "note", "nova", "nude", "null", "numb", "nuts", "oaks", "oars",
    "oath", "obey", "odds", "odor", "ohio", "oils", "oily", "okay", "olds", "omen", "omit", "once",
    "ones", "only", "onto", "oops", "ooze", "opal", "open", "opts", "oral", "orb", "orca", "ores",
    "orig", "oslo", "ours", "oust", "outs", "oval", "oven", "over", "owed", "owes", "owls", "owns",
    "pace", "pack", "pact", "page", "paid", "pain", "pair", "pale", "palm", "pane", "papa", "para",
    "pare", "park", "part", "pass", "past", "path", "pave", "pawn", "pays", "peak", "pear", "peas",
    "peat", "peek", "peel", "peer", "pens", "perm", "peso", "pest", "pete", "pets", "pick", "pies",
    "pigs", "pike", "pile", "pill", "pine", "ping", "pink", "pins", "pint", "pipe", "pity", "plan",
    "play", "plea", "plot", "plow", "plug", "plus", "poem", "poet", "poke", "pole", "poll", "polo",
    "pond", "pony", "pool", "poor", "pope", "pork", "porn", "port", "pose", "post", "pour", "pray",
    "prep", "prey", "prev", "pry", "prix", "prob", "prod", "prof", "prom", "prop", "pros", "puck",
    "pull", "pulp", "pump", "punk", "pupil", "pure", "push", "puts", "quit", "quiz", "race",
    "rack", "raft", "rage", "raid", "rail", "rain", "rake", "rams", "rand", "rang", "rank", "rant",
    "rare", "rash", "rate", "rats", "rave", "rays", "read", "real", "reap", "rear", "redo", "reed",
    "reef", "reel", "reid", "rein", "rely", "remy", "rend", "rene", "rent", "reps", "rest", "rice",
    "rich", "rick", "ride", "rids", "rift", "ring", "riot", "ripe", "rise", "risk", "rita", "rite",
    "road", "roam", "roar", "robe", "rock", "rode", "rods", "role", "roll", "rome", "roof", "room",
    "root", "rope", "rosa", "rose", "ross", "rosy", "roth", "rows", "ruby", "rude", "rugs", "ruin",
    "rule", "rung", "runs", "rush", "rust", "ruth", "ryan", "sack", "safe", "saga", "sage", "said",
    "sail", "sake", "sale", "salt", "same", "sand", "sane", "sang", "sank", "sara", "sash", "save",
    "saws", "says", "scam", "scan", "scar", "seal", "seam", "sear", "seas", "seat", "sect", "seed",
    "seek", "seem", "seen", "sees", "self", "sell", "semi", "send", "sent", "sept", "sera", "seth",
    "sets", "sewer", "shad", "shah", "shed", "shin", "ship", "shit", "shoe", "shop", "shot",
    "show", "shut", "sick", "side", "sigh", "sign", "silk", "sill", "silo", "sing", "sink", "sins",
    "site", "sits", "size", "skid", "skim", "skin", "skip", "skis", "slab", "slam", "slap", "slat",
    "slew", "slid", "slim", "slip", "slit", "slot", "slow", "slug", "slum", "smog", "snap",
    "sneak", "snow", "soak", "soap", "soar", "sock", "soda", "sofa", "soft", "soil", "sold",
    "sole", "solo", "some", "song", "sons", "soon", "soot", "sore", "sort", "soul", "soup", "sour",
    "span", "spar", "spat", "spec", "sped", "spin", "spit", "spot", "spun", "spur", "stab", "stag",
    "star", "stay", "stem", "step", "stew", "stir", "stop", "stow", "stub", "stud", "stun", "such",
    "sued", "suet", "suit", "sulk", "sums", "sung", "sunk", "suns", "sup", "sure", "surf", "swam",
    "swan", "swap", "sway", "swim", "tabs", "tack", "tact", "tags", "tail", "take", "tale", "talk",
    "tall", "tame", "tang", "tank", "tape", "taps", "tara", "task", "tate", "taxi", "teak", "teal",
    "team", "tear", "tech", "teen", "tell", "temp", "tend", "tens", "tent", "term", "tern", "test",
    "text", "thai", "than", "that", "thaw", "thee", "them", "then", "they", "thin", "this", "thus",
    "tick", "tide", "tidy", "tied", "tier", "ties", "tiff", "tile", "till", "tilt", "time", "tina",
    "tine", "ting", "tint", "tiny", "tips", "tire", "toad", "toes", "toil", "told", "toll", "tomb",
    "tome", "tone", "tong", "tony", "took", "tool", "tops", "tore", "torn", "tort", "tory", "toss",
    "tour", "tow", "town", "toys", "tram", "trap", "tray", "tree", "trek", "trim", "trio", "trip",
    "trod", "troy", "true", "tsar", "tub", "tube", "tubs", "tuck", "tuft", "tuna", "tune", "tung",
    "turf", "turn", "tusk", "tutu", "twig", "twin", "twit", "type", "ugly", "ulan", "ulna", "undo",
    "unit", "upon", "urea", "urge", "uric", "urns", "used", "user", "uses", "utah", "vain", "vale",
    "vane", "vars", "vary", "vase", "vast", "veal", "veer", "veil", "vein", "vent", "verb", "vers",
    "very", "vest", "veto", "vets", "vice", "vida", "vied", "view", "vila", "vile", "vine", "visa",
    "void", "volt", "vote", "vows", "wade", "wage", "wail", "wait", "wake", "walk", "wall", "walt",
    "wand", "wang", "want", "ward", "ware", "warm", "warn", "warp", "wars", "wary", "wash", "wasp",
    "wave", "wavy", "ways", "weak", "wear", "webb", "weed", "week", "weep", "weir", "well", "welt",
    "went", "were", "west", "what", "when", "whey", "whim", "whip", "whiz", "whom", "wick", "wide",
    "wife", "wild", "will", "wilt", "wind", "wine", "wing", "wink", "wins", "wipe", "wire", "wiry",
    "wise", "wish", "with", "wits", "woke", "wolf", "womb", "wong", "wood", "wool", "word", "wore",
    "work", "worm", "worn", "wrap", "wren", "writ", "yang", "yard", "yarn", "yawn", "yeah", "year",
    "yell", "yoga", "yoke", "york", "your", "yuan", "yule", "zeal", "zero", "zeus", "zinc", "zone",
    "zoom",
];

#[derive(Debug, Clone)]
pub struct PasswordOptions {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub digits: bool,
    pub symbols: bool,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            length: 20,
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: true,
        }
    }
}

impl PasswordOptions {
    pub fn from_cli(
        length: Option<usize>,
        no_uppercase: bool,
        no_lowercase: bool,
        no_digits: bool,
        no_symbols: bool,
    ) -> Self {
        Self {
            length: length.unwrap_or(20),
            uppercase: !no_uppercase,
            lowercase: !no_lowercase,
            digits: !no_digits,
            symbols: !no_symbols,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PassphraseOptions {
    pub word_count: usize,
}

impl Default for PassphraseOptions {
    fn default() -> Self {
        Self { word_count: 4 }
    }
}

impl PassphraseOptions {
    pub fn from_cli(words: Option<usize>) -> Self {
        Self {
            word_count: words.unwrap_or(4),
        }
    }
}

pub struct PasswordService;

impl PasswordService {
    pub fn validate_generate_options(
        memorable: bool,
        words: Option<usize>,
        length: Option<usize>,
        no_uppercase: bool,
        no_lowercase: bool,
        no_digits: bool,
        no_symbols: bool,
    ) -> Result<()> {
        if memorable
            && (no_uppercase || no_lowercase || no_digits || no_symbols || length.is_some())
        {
            anyhow::bail!("--memorable can only be used with --words option");
        }

        if !memorable && words.is_some() {
            anyhow::bail!("--words option requires --memorable flag");
        }

        Ok(())
    }

    pub fn generate_from_cli(
        memorable: bool,
        words: Option<usize>,
        length: Option<usize>,
        no_uppercase: bool,
        no_lowercase: bool,
        no_digits: bool,
        no_symbols: bool,
    ) -> Result<String> {
        Self::validate_generate_options(
            memorable,
            words,
            length,
            no_uppercase,
            no_lowercase,
            no_digits,
            no_symbols,
        )?;

        if memorable {
            let options = PassphraseOptions::from_cli(words);
            Self::generate_passphrase(&options)
        } else {
            let options = PasswordOptions::from_cli(
                length,
                no_uppercase,
                no_lowercase,
                no_digits,
                no_symbols,
            );
            Self::generate_password(&options)
        }
    }

    pub fn generate_password(options: &PasswordOptions) -> Result<String> {
        if options.length < 8 {
            anyhow::bail!("Password length must be at least 8 characters");
        }
        if options.length > 128 {
            anyhow::bail!("Password length must not exceed 128 characters");
        }

        let charset = Self::build_charset(options)?;

        let password: String = (0..options.length)
            .map(|_| {
                let idx = random_index(charset.len());
                charset[idx]
            })
            .collect();

        Ok(password)
    }

    pub fn generate_passphrase(options: &PassphraseOptions) -> Result<String> {
        if options.word_count < 2 {
            anyhow::bail!("Passphrase must contain at least 2 words");
        }
        if options.word_count > 10 {
            anyhow::bail!("Passphrase must not exceed 10 words");
        }

        let words: Vec<&str> = (0..options.word_count)
            .map(|_| {
                let idx = random_index(WORD_LIST.len());
                WORD_LIST[idx]
            })
            .collect();

        Ok(words.join("-"))
    }

    fn build_charset(options: &PasswordOptions) -> Result<Vec<char>> {
        let mut charset = Vec::new();

        if options.lowercase {
            charset.extend('a'..='z');
        }
        if options.uppercase {
            charset.extend('A'..='Z');
        }
        if options.digits {
            charset.extend('0'..='9');
        }
        if options.symbols {
            charset.extend(SYMBOL_SET.chars());
        }

        if charset.is_empty() {
            anyhow::bail!("Cannot generate password: all character types excluded");
        }

        Ok(charset)
    }
}

fn random_index(max: usize) -> usize {
    let mut bytes = [0u8; 8];
    OsRng.fill_bytes(&mut bytes);
    let value = u64::from_le_bytes(bytes);
    (value as usize) % max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_password_default() {
        let options = PasswordOptions::default();
        let result = PasswordService::generate_password(&options);
        assert!(result.is_ok());
        let password = result.unwrap();
        assert_eq!(password.len(), 20);
    }

    #[test]
    fn test_generate_password_custom_length() {
        let options = PasswordOptions {
            length: 32,
            ..Default::default()
        };
        let result = PasswordService::generate_password(&options);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 32);
    }

    #[test]
    fn test_generate_password_length_validation() {
        let too_short = PasswordOptions {
            length: 4,
            ..Default::default()
        };
        assert!(PasswordService::generate_password(&too_short).is_err());

        let too_long = PasswordOptions {
            length: 200,
            ..Default::default()
        };
        assert!(PasswordService::generate_password(&too_long).is_err());
    }

    #[test]
    fn test_empty_charset_error() {
        let options = PasswordOptions {
            length: 20,
            uppercase: false,
            lowercase: false,
            digits: false,
            symbols: false,
        };
        let result = PasswordService::generate_password(&options);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("all character types excluded"));
    }

    #[test]
    fn test_generate_passphrase() {
        let options = PassphraseOptions { word_count: 4 };
        let result = PasswordService::generate_passphrase(&options);
        assert!(result.is_ok());
        let passphrase = result.unwrap();
        assert_eq!(passphrase.split('-').count(), 4);
    }

    #[test]
    fn test_passphrase_word_count_validation() {
        let too_few = PassphraseOptions { word_count: 1 };
        assert!(PasswordService::generate_passphrase(&too_few).is_err());

        let too_many = PassphraseOptions { word_count: 20 };
        assert!(PasswordService::generate_passphrase(&too_many).is_err());
    }

    #[test]
    fn test_password_options_from_cli() {
        let options = PasswordOptions::from_cli(Some(32), true, false, true, false);
        assert_eq!(options.length, 32);
        assert!(!options.uppercase);
        assert!(options.lowercase);
        assert!(!options.digits);
        assert!(options.symbols);
    }

    #[test]
    fn test_validate_generate_options() {
        assert!(PasswordService::validate_generate_options(
            true,
            None,
            Some(20),
            false,
            false,
            false,
            false
        )
        .is_err());

        assert!(PasswordService::validate_generate_options(
            false,
            Some(4),
            None,
            false,
            false,
            false,
            false
        )
        .is_err());

        assert!(PasswordService::validate_generate_options(
            true,
            Some(4),
            None,
            false,
            false,
            false,
            false
        )
        .is_ok());
    }

    #[test]
    fn test_generate_from_cli() {
        let result =
            PasswordService::generate_from_cli(false, None, Some(16), false, false, false, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 16);

        let result =
            PasswordService::generate_from_cli(true, Some(3), None, false, false, false, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().split('-').count(), 3);
    }
}
