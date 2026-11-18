//! Board games: year, designer, player count, minimum age and playing time.
//!
//! "Agricola", "Axis & Allies vs. Risk", "how many players are needed for
//! Risk?", "board games with minimum age >= 5", "board games for 2 players".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

struct Game {
    /// display name; other names after ';'
    names: &'static str,
    /// answers a bare query (the name is not also a common word)
    bare: bool,
    year: &'static str,
    designer: &'static str,
    min_players: u32,
    max_players: u32,
    min_age: u32,
    time: &'static str,
    kind: &'static str,
    note: &'static str,
}

macro_rules! g {
    ($n:expr, $b:expr, $y:expr, $d:expr, $p0:expr, $p1:expr, $a:expr, $t:expr, $k:expr, $note:expr) => {
        Game { names: $n, bare: $b, year: $y, designer: $d, min_players: $p0, max_players: $p1, min_age: $a, time: $t, kind: $k, note: $note }
    };
}

const GAMES: &[Game] = &[
    g!("Risk;risk board game;game of risk", false, "1957 (as La Conquête du Monde)", "Albert Lamorisse", 2, 6, 10, "1–8 hours", "war strategy, area control", "players conquer territories on a map of the world with armies and dice"),
    g!("Axis & Allies;axis and allies;axis & allies;axis allies", true, "1981 (Milton Bradley edition 1984)", "Larry Harris", 2, 5, 12, "3–6 hours", "World War II strategy", "the Axis powers against the Allies in spring 1942"),
    g!("Agricola", true, "2007", "Uwe Rosenberg", 1, 5, 12, "30–150 minutes", "worker placement, farming", "players develop a 17th-century farm over 14 rounds"),
    g!("Monopoly;monopoly board game;game of monopoly", false, "1935 (Parker Brothers; based on The Landlord's Game, 1904)", "Charles Darrow, after Lizzie Magie", 2, 8, 8, "1–3 hours", "economic, roll and move", "players buy, rent and trade properties to bankrupt their opponents"),
    g!("Scrabble;scrabble board game", false, "1938 (as Criss-Crosswords; Scrabble from 1948)", "Alfred Mosher Butts", 2, 4, 8, "about 90 minutes", "word game", "letter tiles placed crossword-style on a 15 × 15 board; 100 tiles"),
    g!("Catan;settlers of catan;the settlers of catan", true, "1995", "Klaus Teuber", 3, 4, 10, "60–120 minutes", "trading, resource management", "players build settlements, cities and roads on a hexagonal island; 10 victory points win"),
    g!("Carcassonne", true, "2000", "Klaus-Jürgen Wrede", 2, 5, 7, "30–45 minutes", "tile placement", "players lay landscape tiles and place followers on roads, cities and fields"),
    g!("Ticket to Ride", true, "2004", "Alan R. Moon", 2, 5, 8, "30–60 minutes", "route building, set collection", "players claim railway routes across North America"),
    g!("Clue;cluedo;clue board game", false, "1949", "Anthony E. Pratt", 2, 6, 8, "about 45 minutes", "deduction", "players deduce the murderer, the weapon and the room"),
    g!("Stratego", true, "1946 (after L'Attaque, 1909)", "Jacques Johan Mogendorff", 2, 2, 8, "about 45 minutes", "war strategy, hidden information", "each player has 40 pieces whose ranks are hidden; capture the opponent's flag"),
    g!("Candy Land;candyland", true, "1949", "Eleanor Abbott", 2, 4, 3, "15–30 minutes", "children's race", "no reading or decisions needed; moves are drawn from a deck of colour cards"),
    g!("Chutes and Ladders;snakes and ladders", true, "ancient India (Milton Bradley Chutes and Ladders 1943)", "traditional", 2, 6, 3, "about 30 minutes", "children's race", "pure chance: ladders move a token up and chutes (snakes) down"),
    g!("Sorry!;sorry board game", false, "1934", "William Henry Storey", 2, 4, 6, "about 30 minutes", "race", "a cross-and-circle race game in which players bump opponents back to start"),
    g!("Trouble;trouble board game", false, "1965", "Frederick Kroll", 2, 4, 5, "about 30 minutes", "race", "moves are rolled with the Pop-O-Matic die in a plastic dome"),
    g!("The Game of Life;game of life board game;life board game", false, "1860 (The Checkered Game of Life; modern version 1960)", "Milton Bradley; modern version Reuben Klamer", 2, 6, 8, "about 60 minutes", "roll and move", "players travel through life events, careers and families; not Conway's cellular automaton"),
    g!("Battleship;battleship board game", false, "1967 (Milton Bradley; earlier as a pencil-and-paper game)", "traditional", 2, 2, 7, "about 30 minutes", "guessing", "players guess the grid coordinates of each other's hidden ships"),
    g!("Connect Four;connect 4", true, "1974", "Howard Wexler and Ned Strongin", 2, 2, 6, "about 10 minutes", "abstract", "drop discs into a 7 × 6 grid to make four in a row; solved: the first player wins with perfect play"),
    g!("Mouse Trap;mousetrap board game", false, "1963", "Marvin Glass", 2, 4, 6, "about 30 minutes", "children's", "players build a Rube Goldberg-style mouse trap"),
    g!("Pandemic;pandemic board game", false, "2008", "Matt Leacock", 2, 4, 8, "about 45 minutes", "cooperative", "players work together to cure four diseases spreading across the world"),
    g!("Twilight Struggle", true, "2005", "Ananda Gupta and Jason Matthews", 2, 2, 13, "2–3 hours", "card-driven Cold War strategy", "the United States against the Soviet Union, 1945–1989"),
    g!("Gloomhaven", true, "2017", "Isaac Childres", 1, 4, 14, "60–120 minutes", "cooperative campaign, dungeon crawl", "a legacy-style campaign of about 95 scenarios"),
    g!("Terraforming Mars", true, "2016", "Jacob Fryxelius", 1, 5, 12, "about 2 hours", "engine building", "corporations raise Mars's temperature and oxygen and place oceans"),
    g!("Dominion;dominion card game", false, "2008", "Donald X. Vaccarino", 2, 4, 13, "about 30 minutes", "deck building", "the game that started the deck-building genre"),
    g!("Azul;azul board game", false, "2017", "Michael Kiesling", 2, 4, 8, "30–45 minutes", "abstract, tile drafting", "players draft Portuguese-style tiles to decorate a palace wall"),
    g!("Diplomacy;diplomacy board game", false, "1959", "Allan B. Calhamer", 2, 7, 12, "about 6 hours", "negotiation, war strategy", "the great powers of Europe before World War I; best with 7 players; no dice"),
    g!("Trivial Pursuit", true, "1981", "Chris Haney and Scott Abbott", 2, 6, 12, "about 90 minutes", "trivia", "answer questions in six categories to collect six wedges; teams allow more players"),
    g!("Operation;operation board game", false, "1965", "John Spinello", 1, 6, 6, "about 15 minutes", "dexterity", "remove ailments with tweezers without touching the edge of the cavity"),
    g!("Hungry Hungry Hippos", true, "1978", "Fred Kroll", 2, 4, 4, "about 5 minutes", "children's action", "players press levers to make their hippos gobble marbles"),
    g!("Guess Who?;guess who", true, "1979", "Theo and Ora Coster", 2, 2, 6, "about 20 minutes", "deduction", "yes-or-no questions narrow down the opponent's hidden face"),
    g!("Codenames", true, "2015", "Vlaada Chvátil", 2, 8, 14, "about 15 minutes", "party, word association", "two teams' spymasters give one-word clues to grid words"),
    g!("Splendor", true, "2014", "Marc André", 2, 4, 10, "about 30 minutes", "engine building", "players collect gem tokens to buy development cards"),
    g!("7 Wonders;seven wonders board game", true, "2010", "Antoine Bauza", 2, 7, 10, "about 30 minutes", "card drafting, civilization", "simultaneous card drafting over three ages"),
    g!("Puerto Rico;puerto rico board game", false, "2002", "Andreas Seyfarth", 2, 5, 12, "90–150 minutes", "role selection, economic", "colonial plantation and shipping economy"),
    g!("Power Grid", true, "2004", "Friedemann Friese", 2, 6, 12, "about 2 hours", "economic, auction", "players buy power plants and fuel and connect cities"),
    g!("Wingspan;wingspan board game", false, "2019", "Elizabeth Hargrave", 1, 5, 10, "40–70 minutes", "engine building", "players attract birds to their wildlife preserves; 170 bird cards"),
    g!("Mastermind;mastermind board game", false, "1970", "Mordecai Meirowitz", 2, 2, 8, "about 20 minutes", "code breaking", "guess a hidden code of four coloured pegs; any code can be found in at most five guesses"),
    g!("Parcheesi;pachisi;ludo", true, "ancient India (Parcheesi trademark 1867)", "traditional", 2, 4, 6, "about 45 minutes", "race", "a cross-and-circle race game from the Indian game pachisi"),
    g!("chess", true, "about the 6th century (from the Indian game chaturanga)", "traditional", 2, 2, 6, "10 minutes to several hours", "abstract strategy", "8 × 8 board, 16 pieces per side; win by checkmating the king"),
    g!("checkers;draughts;english draughts", true, "about the 12th century in its modern form", "traditional", 2, 2, 6, "about 30 minutes", "abstract strategy", "8 × 8 board, 12 pieces per side; solved in 2007: perfect play is a draw"),
    g!("backgammon", true, "about 5,000 years ago (the Royal Game of Ur family); modern rules 17th century", "traditional", 2, 2, 8, "about 30 minutes", "race, dice", "24 points, 15 checkers per side, doubling cube"),
    g!("go;go board game;game of go;baduk;weiqi", false, "over 2,500 years ago in China", "traditional", 2, 2, 8, "30 minutes to several hours", "abstract strategy, territory", "19 × 19 board, black and white stones; win by surrounding more territory"),
    g!("Yahtzee", true, "1956", "Edwin S. Lowe (from the folk game yacht)", 1, 10, 8, "about 30 minutes", "dice", "five dice, up to three rolls a turn, 13 scoring categories"),
];

fn names(g: &Game) -> impl Iterator<Item = &'static str> {
    g.names.split(';')
}

fn norm(s: &str) -> String {
    let s = s.trim().trim_start_matches("the ").replace(" and ", " & ").replace('!', "").replace('?', "").replace('.', "");
    s.trim().to_string()
}

/// "risk", "the game risk", "risk board game"; the flag says the query is
/// clearly about the game
fn find(s: &str) -> Option<(&'static Game, bool)> {
    let s = s.trim();
    let key = norm(s);
    if let Some((g, n)) = GAMES.iter().find_map(|g| names(g).find(|n| norm(&n.to_lowercase()) == key).map(|n| (g, n))) {
        return Some((g, g.bare || n.contains("game")));
    }
    let m = re!(r"^(?:the )?(?:board ?game|game) (?:of )?(.+)$|^(.+?) (?:board ?game|game)$").captures(s)?;
    let key = norm(m.get(1).or(m.get(2))?.as_str());
    GAMES.iter().find(|g| names(g).any(|n| norm(&n.to_lowercase()) == key)).map(|g| (g, true))
}

fn players(g: &Game) -> String {
    if g.min_players == g.max_players {
        format!("{}", g.min_players)
    } else {
        format!("{}–{}", g.min_players, g.max_players)
    }
}

fn title(g: &Game) -> &'static str {
    names(g).next().unwrap()
}

fn card(g: &Game) -> Answer {
    Answer::new(format!("{} (board game)", title(g)), vec![
        pod("Input", "Input interpretation", text(format!("{} (board game)", title(g)))),
        primary(pod("Result", "Basic information", text(format!("{} players | ages {}+ | {}", players(g), g.min_age, g.time)))),
        pod("Table", "Properties", table(None, vec![
            vec![text("first published"), text(g.year)],
            vec![text("designer"), text(g.designer)],
            vec![text("players"), text(players(g))],
            vec![text("minimum age"), text(format!("{} years", g.min_age))],
            vec![text("playing time"), text(g.time)],
            vec![text("genre"), text(g.kind)],
            vec![text("description"), text(g.note)],
        ])),
    ])
}

fn compare(list: &[&'static Game]) -> Answer {
    let mut head = vec![""];
    head.extend(list.iter().map(|g| title(g)));
    let row = |label: &str, f: &dyn Fn(&Game) -> String| {
        let mut r = vec![text(label)];
        r.extend(list.iter().map(|g| text(f(g))));
        r
    };
    let rows = vec![
        row("first published", &|g| g.year.to_string()),
        row("designer", &|g| g.designer.to_string()),
        row("players", &|g| players(g)),
        row("minimum age", &|g| format!("{}+", g.min_age)),
        row("playing time", &|g| g.time.to_string()),
        row("genre", &|g| g.kind.to_string()),
    ];
    let names: Vec<&str> = list.iter().map(|g| title(g)).collect();
    Answer::new("board game comparison", vec![
        pod("Input", "Input interpretation", text(format!("{} (board games)", names.join(" vs. ")))),
        primary(pod("Result", "Comparison", text(list.iter().map(|g| format!("{}: {} players, ages {}+, {}", title(g), players(g), g.min_age, g.time)).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Properties", table(Some(head), rows)),
    ])
}

fn listing(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:(?:list of |all )?board ?games|games)\s+(?:with |for )?(?:(minimum age|min age|age|ages)\s*(>=|<=|>|<|=|of|at least|at most|under|over)?\s*(\d+)\+?|(\d+)\s+players?)$").captures(l)?;
    let (desc, sel): (String, Vec<&Game>) = if let Some(n) = m.get(4) {
        let p: u32 = n.as_str().parse().ok()?;
        (format!("board games for {p} players"), GAMES.iter().filter(|g| g.min_players <= p && p <= g.max_players).collect())
    } else {
        let v: u32 = m[3].parse().ok()?;
        let op = m.get(2).map(|x| x.as_str()).unwrap_or("=");
        let f: Box<dyn Fn(u32) -> bool> = match op {
            ">=" | "at least" => Box::new(move |a| a >= v),
            ">" | "over" => Box::new(move |a| a > v),
            "<=" | "at most" => Box::new(move |a| a <= v),
            "<" | "under" => Box::new(move |a| a < v),
            _ => Box::new(move |a| a == v),
        };
        (format!("board games with minimum age {op} {v}"), GAMES.iter().filter(|g| f(g.min_age)).collect())
    };
    if sel.is_empty() {
        return None;
    }
    let rows: Vec<Vec<Content>> = sel.iter().map(|g| vec![text(title(g)), text(players(g)), text(format!("{}+", g.min_age)), text(g.time)]).collect();
    Some(Answer::new("board games", vec![
        pod("Input", "Input interpretation", text(desc)),
        primary(pod("Result", "Result", text(format!("{} games: {}", sel.len(), sel.iter().map(|g| title(g)).collect::<Vec<_>>().join(", "))))),
        pod("Table", "Games", table(Some(vec!["game", "players", "minimum age", "playing time"]), rows)),
        pod("Note", "Scope", text("among about 40 well-known board games in Noesis's built-in list")),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    if let Some(a) = listing(l) {
        return Some(a);
    }
    // "how many players are needed for Risk" (cleaned to "players are needed
    // for risk"), "players in Catan"
    if let Some(m) = re!(r"^(?:how many |number of )?(?:players|people)(?: (?:are|do you))?(?: (?:needed|need|required|can play))? (?:for|to play|in|of)\s+(.+)$").captures(l) {
        let (g, _) = find(&m[1])?;
        return Some(Answer::new(format!("{} (board game)", title(g)), vec![
            pod("Input", "Input interpretation", text(format!("{} (board game) | number of players", title(g)))),
            primary(pod("Result", "Number of players", text(format!("{} players", players(g))))),
            pod("Table", "Properties", table(None, vec![vec![text("minimum age"), text(format!("{} years", g.min_age))], vec![text("playing time"), text(g.time)]])),
        ]));
    }
    // comparison: "Axis & Allies vs. Risk"
    if re!(r"\bvs\.?\b|\bversus\b").is_match(l) {
        let parts: Vec<&str> = re!(r"\s+(?:vs\.?|versus)\s+").split(l).collect();
        if parts.len() >= 2 {
            let found: Option<Vec<(&Game, bool)>> = parts.iter().map(|p| find(p)).collect();
            let found = found?;
            if !found.iter().any(|f| f.1) {
                return None;
            }
            let list: Vec<&Game> = found.iter().map(|f| f.0).collect();
            return Some(compare(&list));
        }
    }
    let (g, ok) = find(l)?;
    ok.then(|| card(g))
}
