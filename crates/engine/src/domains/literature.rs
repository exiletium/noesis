//! Notable texts: novels, plays, poems, scripture and founding documents,
//! with authors, dates, original languages, structure (chapters, acts and
//! scenes) and approximate lengths; works by author; a few famous verses.

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

struct Work {
    names: Vec<String>,
    author: String,
    year: String,
    language: String,
    form: String,
    chapters: Option<u32>,
    /// scenes in each act, for plays
    scenes: Vec<u32>,
    words: Option<u32>,
    note: String,
}

impl Work {
    fn title(&self) -> &str {
        &self.names[0]
    }
}

// titles | author | first published | original language | form | chapters (or books, cantos, poems) | scenes per act | approximate words | note
const DATA: &str = "\
Moby-Dick;moby dick;moby-dick or the whale|Herman Melville|1851|English|novel|135||209000|plus an epilogue; first published in London as The Whale
The Grapes of Wrath;grapes of wrath|John Steinbeck|1939|English|novel|30||169000|Pulitzer Prize 1940
Dune|Frank Herbert|1965|English|novel|||188000|three books: Dune, Muad'Dib, The Prophet; Hugo and Nebula awards
Naked Lunch|William S. Burroughs|1959|English|novel|||73000|first published in Paris by Olympia Press
1st to Die;first to die|James Patterson|2001|English|novel|126||85000|first book of the Women's Murder Club series
War and Peace|Leo Tolstoy|1869|Russian (with substantial passages in French)|novel|361||587000|serialized 1865–1867; four volumes and a two-part epilogue
Lolita|Vladimir Nabokov|1955|English|novel|69||112000|two parts; first published in Paris by Olympia Press; Nabokov later translated it into Russian (1967)
On the Origin of Species;origin of species;the origin of species|Charles Darwin|1859|English|non-fiction (natural history)|14||150000|full title: On the Origin of Species by Means of Natural Selection
Walden;walden or life in the woods|Henry David Thoreau|1854|English|non-fiction (memoir, essays)|18||114000|
The Republic;republic;plato's republic|Plato|c. 375 BC|Ancient Greek|philosophical dialogue|10||118000|ten books; a Socratic dialogue on justice and the ideal state
Leaves of Grass|Walt Whitman|1855|English|poetry collection|12||| first edition had 12 untitled poems; Whitman revised and expanded it until 1892 (the 'deathbed edition', nearly 400 poems)
Shakespeare's Sonnets;sonnets;shakespeare's sonnets|William Shakespeare|1609|English|poetry (sonnet sequence)|154||17500|published by Thomas Thorpe in 1609
Frankenstein;frankenstein or the modern prometheus|Mary Shelley|1818|English|novel|24||75000|published anonymously in 1818; revised edition 1831
Adventures of Huckleberry Finn;huckleberry finn;huck finn;the adventures of huckleberry finn|Mark Twain|1884|English|novel|43||110000|published in the UK in December 1884 and the US in February 1885
Great Expectations|Charles Dickens|1861|English|novel|59||183000|serialized 1860–1861
Pride and Prejudice|Jane Austen|1813|English|novel|61||122000|
Nineteen Eighty-Four;1984;nineteen eighty four|George Orwell|1949|English|novel|23||89000|three parts
The Great Gatsby;great gatsby|F. Scott Fitzgerald|1925|English|novel|9||47000|
To Kill a Mockingbird|Harper Lee|1960|English|novel|31||100000|Pulitzer Prize 1961
Don Quixote|Miguel de Cervantes|1605|Spanish|novel|126||345000|second part 1615
The Odyssey;odyssey|Homer|c. 8th century BC|Ancient Greek (Homeric)|epic poem|24||134000|24 books, about 12,100 lines of dactylic hexameter
The Iliad;iliad|Homer|c. 8th century BC|Ancient Greek (Homeric)|epic poem|24||156000|24 books, about 15,700 lines
The Divine Comedy;divine comedy|Dante Alighieri|1320|Italian (Tuscan)|epic poem|100||| 100 cantos in three parts: Inferno, Purgatorio, Paradiso
Crime and Punishment|Fyodor Dostoevsky|1866|Russian|novel|41||211000|six parts and an epilogue
Ulysses|James Joyce|1922|English|novel|18||265000|18 episodes
Jane Eyre|Charlotte Brontë|1847|English|novel|38||183000|
Wuthering Heights|Emily Brontë|1847|English|novel|34||107000|
The Catcher in the Rye;catcher in the rye|J. D. Salinger|1951|English|novel|26||73000|
Hamlet|William Shakespeare|c. 1600|English (Early Modern)|tragedy (play)||5,2,4,7,2|30000|Shakespeare's longest play
Macbeth|William Shakespeare|c. 1606|English (Early Modern)|tragedy (play)||7,4,6,3,8|17000|Shakespeare's shortest tragedy
Othello|William Shakespeare|c. 1603|English (Early Modern)|tragedy (play)||3,3,4,3,2|26000|
King Lear|William Shakespeare|c. 1606|English (Early Modern)|tragedy (play)||5,4,7,7,3|25000|
Romeo and Juliet|William Shakespeare|c. 1595|English (Early Modern)|tragedy (play)||5,6,5,5,3|24000|
Antony and Cleopatra;anthony and cleopatra|William Shakespeare|c. 1607|English (Early Modern)|tragedy (play)||5,7,13,15,2|24000|
Julius Caesar|William Shakespeare|c. 1599|English (Early Modern)|tragedy (play)||3,4,3,3,5|19000|
The Tempest;tempest|William Shakespeare|c. 1611|English (Early Modern)|comedy (romance)||2,2,3,1,1|16000|
A Midsummer Night's Dream;midsummer night's dream|William Shakespeare|c. 1595|English (Early Modern)|comedy (play)||2,2,2,2,1|16000|
Twelfth Night|William Shakespeare|c. 1601|English (Early Modern)|comedy (play)||5,5,4,3,1|19000|
King John|William Shakespeare|c. 1596|English (Early Modern)|history (play)||1,1,4,3,7|20000|
Richard II;king richard ii;king richard the second|William Shakespeare|c. 1595|English (Early Modern)|history (play)||4,4,4,1,6|22000|
Richard III;king richard iii|William Shakespeare|c. 1593|English (Early Modern)|history (play)||4,4,7,5,5|29000|
Henry V;king henry v|William Shakespeare|c. 1599|English (Early Modern)|history (play)||2,4,7,8,2|26000|
Hedda Gabler|Henrik Ibsen|1890|Norwegian (Dano-Norwegian)|play (drama)||1,1,1,1|21000|four acts
A Doll's House;doll's house|Henrik Ibsen|1879|Norwegian (Dano-Norwegian)|play (drama)||1,1,1|24000|three acts
Peer Gynt|Henrik Ibsen|1867|Norwegian (Dano-Norwegian)|verse play||1,1,1,1,1|| five acts
Ghosts|Henrik Ibsen|1881|Norwegian (Dano-Norwegian)|play (drama)||1,1,1|| three acts
An Enemy of the People|Henrik Ibsen|1882|Norwegian (Dano-Norwegian)|play (drama)||1,1,1,1,1|| five acts
The Wild Duck;wild duck|Henrik Ibsen|1884|Norwegian (Dano-Norwegian)|play (drama)||1,1,1,1,1|| five acts
The Bible;bible;holy bible|various authors|c. 1200 BC – AD 100 (composition); canon fixed in the 4th century|Hebrew, Aramaic and Koine Greek|scripture|1189||783137|66 books in the Protestant canon (39 Old Testament, 27 New Testament); 31,102 verses; word count is for the King James Version
The Pentateuch;pentateuch;torah|traditionally Moses|c. 6th–5th century BC (final form)|Hebrew|scripture|187||80000|five books: Genesis, Exodus, Leviticus, Numbers, Deuteronomy; 5,852 verses
The Quran;quran;koran|revealed to Muhammad (Islamic tradition)|c. 610–632|Classical Arabic|scripture|114||77000|114 surahs; 6,236 verses
Constitution of the United States;us constitution;the us constitution;united states constitution;the constitution|Framers of the Constitutional Convention|1787|English|founding document|7||4543|seven articles; 4,543 words in the original (including signatures); about 7,600 words with the 27 amendments
Declaration of Independence;the declaration of independence|Thomas Jefferson (principal author)|1776|English|founding document|||1458|adopted July 4, 1776
Gettysburg Address;the gettysburg address|Abraham Lincoln|1863|English|speech|||272|delivered November 19, 1863 (Bliss copy: 272 words)
The Analects;analects|disciples of Confucius|c. 475–221 BC (compiled)|Classical Chinese|philosophy|20||| 20 books of sayings attributed to Confucius
";

fn works() -> &'static Vec<Work> {
    static W: OnceLock<Vec<Work>> = OnceLock::new();
    W.get_or_init(|| {
        DATA.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| {
                let f: Vec<&str> = l.split('|').collect();
                if f.len() != 9 {
                    return None;
                }
                Some(Work {
                    names: f[0].split(';').map(String::from).collect(),
                    author: f[1].into(),
                    year: f[2].into(),
                    language: f[3].into(),
                    form: f[4].into(),
                    chapters: f[5].parse().ok(),
                    scenes: f[6].split(',').filter_map(|x| x.trim().parse().ok()).collect(),
                    words: f[7].parse().ok(),
                    note: f[8].trim().into(),
                })
            })
            .collect()
    })
}

/// (author, aliases, works with years)
const AUTHORS: &[(&str, &[&str], &str, &[(&str, &str)])] = &[
    ("Henrik Ibsen", &["ibsen", "henrik ibsen"], "plays", &[("Brand", "1865"), ("Peer Gynt", "1867"), ("Pillars of Society", "1877"), ("A Doll's House", "1879"), ("Ghosts", "1881"), ("An Enemy of the People", "1882"), ("The Wild Duck", "1884"), ("Rosmersholm", "1886"), ("The Lady from the Sea", "1888"), ("Hedda Gabler", "1890"), ("The Master Builder", "1892"), ("Little Eyolf", "1894"), ("John Gabriel Borkman", "1896"), ("When We Dead Awaken", "1899")]),
    ("Sinclair Lewis", &["sinclair lewis"], "books", &[("Our Mr. Wrenn", "1914"), ("Main Street", "1920"), ("Babbitt", "1922"), ("Arrowsmith", "1925"), ("Elmer Gantry", "1927"), ("Dodsworth", "1929"), ("Ann Vickers", "1933"), ("It Can't Happen Here", "1935"), ("Cass Timberlane", "1945"), ("Kingsblood Royal", "1947")]),
    ("Confucius", &["confucius", "kongzi", "kong fuzi"], "books", &[("The Analects (compiled by his disciples)", "c. 475–221 BC"), ("Spring and Autumn Annals (traditionally attributed)", "c. 5th century BC"), ("Book of Songs (Shijing, traditionally edited)", "c. 1000–600 BC"), ("Book of Documents (Shujing, traditionally edited)", ""), ("Book of Rites (Liji)", ""), ("I Ching (Book of Changes; commentaries attributed)", "")]),
    ("William Shakespeare", &["shakespeare", "william shakespeare"], "plays", &[("Romeo and Juliet", "c. 1595"), ("A Midsummer Night's Dream", "c. 1595"), ("The Merchant of Venice", "c. 1597"), ("Much Ado About Nothing", "c. 1599"), ("Julius Caesar", "c. 1599"), ("Henry V", "c. 1599"), ("Hamlet", "c. 1600"), ("Twelfth Night", "c. 1601"), ("Othello", "c. 1603"), ("King Lear", "c. 1606"), ("Macbeth", "c. 1606"), ("Antony and Cleopatra", "c. 1607"), ("The Tempest", "c. 1611")]),
    ("Charles Dickens", &["dickens", "charles dickens"], "books", &[("The Pickwick Papers", "1836"), ("Oliver Twist", "1838"), ("A Christmas Carol", "1843"), ("David Copperfield", "1850"), ("Bleak House", "1853"), ("Hard Times", "1854"), ("A Tale of Two Cities", "1859"), ("Great Expectations", "1861"), ("Our Mutual Friend", "1865")]),
    ("Mark Twain", &["mark twain", "twain"], "books", &[("The Innocents Abroad", "1869"), ("Roughing It", "1872"), ("The Adventures of Tom Sawyer", "1876"), ("The Prince and the Pauper", "1881"), ("Life on the Mississippi", "1883"), ("Adventures of Huckleberry Finn", "1884"), ("A Connecticut Yankee in King Arthur's Court", "1889"), ("Pudd'nhead Wilson", "1894")]),
    ("Jane Austen", &["jane austen", "austen"], "books", &[("Sense and Sensibility", "1811"), ("Pride and Prejudice", "1813"), ("Mansfield Park", "1814"), ("Emma", "1815"), ("Northanger Abbey", "1817"), ("Persuasion", "1817")]),
    ("Leo Tolstoy", &["tolstoy", "leo tolstoy"], "books", &[("War and Peace", "1869"), ("Anna Karenina", "1878"), ("The Death of Ivan Ilyich", "1886"), ("Resurrection", "1899")]),
];

/// A few well-known verses (King James Version, public domain).
const VERSES: &[(&str, &str)] = &[
    ("1 corinthians 13:11", "When I was a child, I spake as a child, I understood as a child, I thought as a child: but when I became a man, I put away childish things."),
    ("1 corinthians 13:4", "Charity suffereth long, and is kind; charity envieth not; charity vaunteth not itself, is not puffed up,"),
    ("1 corinthians 13:13", "And now abideth faith, hope, charity, these three; but the greatest of these is charity."),
    ("john 3:16", "For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life."),
    ("genesis 1:1", "In the beginning God created the heaven and the earth."),
    ("psalm 23:1", "The LORD is my shepherd; I shall not want."),
    ("john 11:35", "Jesus wept."),
    ("ecclesiastes 3:1", "To every thing there is a season, and a time to every purpose under the heaven:"),
    ("matthew 5:9", "Blessed are the peacemakers: for they shall be called the children of God."),
    ("romans 8:28", "And we know that all things work together for good to them that love God, to them who are the called according to his purpose."),
];

fn find(s: &str) -> Option<&'static Work> {
    let s = s.trim().trim_end_matches("'s").trim_matches('"').trim();
    let s = s.strip_prefix("shakespeare's ").filter(|x| !x.starts_with("sonnet")).unwrap_or(s);
    works().iter().find(|w| w.names.iter().any(|n| n.to_lowercase() == s)).or_else(|| {
        let t = s.trim_start_matches("the ");
        works().iter().find(|w| w.names.iter().any(|n| n.to_lowercase().trim_start_matches("the ") == t))
    })
}

fn split(s: &str) -> Option<Vec<&'static Work>> {
    let parts: Vec<&str> = re!(r"\s*(?:,\s*and\s+|,|\bvs\.?\b|\bversus\b|&|;)\s*").split(s).filter(|x| !x.trim().is_empty()).collect();
    // "Antony and Cleopatra", "Romeo and Juliet": only split on "and" when every part is a title
    let ws: Option<Vec<&Work>> = parts.iter().map(|p| find(p)).collect();
    if ws.is_some() {
        return ws;
    }
    let parts: Vec<&str> = re!(r"\s*(?:,\s*and\s+|,|\band\b|\bvs\.?\b|\bversus\b|&|;)\s*").split(s).filter(|x| !x.trim().is_empty()).collect();
    parts.iter().map(|p| find(p)).collect()
}

fn props(w: &Work) -> Content {
    let mut r = vec![vec![text("author"), text(w.author.clone())], vec![text("first published"), text(w.year.clone())], vec![text("original language"), text(w.language.clone())], vec![text("form"), text(w.form.clone())]];
    if let Some(c) = w.chapters {
        r.push(vec![text(if w.form.contains("scripture") { "chapters" } else if w.form.contains("poem") && !w.form.contains("collection") { "books or cantos" } else if w.form.contains("document") { "articles" } else { "chapters" }), text(c.to_string())]);
    }
    if !w.scenes.is_empty() {
        r.push(vec![text("acts"), text(w.scenes.len().to_string())]);
        if w.scenes.iter().any(|&s| s > 1) {
            r.push(vec![text("scenes"), text(format!("{} ({})", w.scenes.iter().sum::<u32>(), w.scenes.iter().enumerate().map(|(i, s)| format!("act {}: {s}", i + 1)).collect::<Vec<_>>().join(", ")))]);
        }
    }
    if let Some(n) = w.words {
        r.push(vec![text("length"), text(format!("about {} words", group(n)))]);
    }
    if !w.note.is_empty() {
        r.push(vec![text("note"), text(w.note.clone())]);
    }
    table(None, r)
}

fn group(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn card(w: &Work) -> Answer {
    Answer::new("text", vec![
        pod("Input", "Input interpretation", text(format!("{} ({})", w.title(), w.form))),
        primary(pod("Result", "Basic information", text(format!("{} by {}, first published {}", w.form, w.author, w.year)))),
        pod("Properties", "Properties", props(w)),
    ])
}

fn value(w: &Work, what: &str, act: Option<usize>) -> Option<String> {
    Some(match what {
        "chapters" => w.chapters?.to_string(),
        "acts" => {
            if w.scenes.is_empty() {
                return None;
            }
            w.scenes.len().to_string()
        }
        "scenes" => match act {
            Some(a) => w.scenes.get(a.checked_sub(1)?)?.to_string(),
            None => {
                if w.scenes.is_empty() {
                    return None;
                }
                w.scenes.iter().sum::<u32>().to_string()
            }
        },
        "words" => format!("about {}", group(w.words?)),
        "language" => w.language.clone(),
        "published" => w.year.clone(),
        "author" => w.author.clone(),
        _ => return None,
    })
}

fn answer_many(ws: &[&Work], what: &str, act: Option<usize>, label: &str) -> Option<Answer> {
    let vals: Vec<(String, String)> = ws.iter().map(|w| (w.title().to_string(), value(w, what, act).unwrap_or_else(|| "(not available)".into()))).collect();
    if vals.iter().all(|v| v.1 == "(not available)") {
        return None;
    }
    let main = if vals.len() == 1 { vals[0].1.clone() } else { vals.iter().map(|(t, v)| format!("{t}: {v}")).collect::<Vec<_>>().join(" | ") };
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{label} of {}", ws.iter().map(|w| w.title()).collect::<Vec<_>>().join(", ")))), primary(pod("Result", label, text(main)))];
    if ws.len() == 1 {
        pods.push(pod("Properties", "Properties", props(ws[0])));
    } else {
        pods.push(pod("Table", "Comparison", table(Some(vec!["title", label]), vals.into_iter().map(|(t, v)| vec![text(t), text(v)]).collect())));
    }
    Some(Answer::new("text", pods))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    // verses: "1 Corinthians 13:11"
    if let Some(v) = VERSES.iter().find(|v| v.0 == l.trim_start_matches("the verse ").trim_end_matches(" kjv")) {
        let mut c = v.0.chars();
        let title = c.next().map(|x| x.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default();
        return Some(Answer::new("verse", vec![pod("Input", "Input interpretation", text(format!("{title} (King James Version)"))), primary(pod("Result", "Text", text(v.1)))]));
    }
    // "books by Sinclair Lewis", "plays by Henrik Ibsen", "notable books written by Confucius"
    if let Some(m) = re!(r"^(?:notable |famous |list of )?(books|novels|plays|works|writings) (?:written )?by (.+)$").captures(l) {
        let who = m[2].trim();
        let a = AUTHORS.iter().find(|a| a.1.contains(&who))?;
        let t = a.3.iter().map(|(w, y)| vec![text(*w), text(*y)]).collect();
        return Some(Answer::new("works", vec![pod("Input", "Input interpretation", text(format!("{} by {}", &m[1], a.0))), primary(pod("Result", "Result", text(a.3.iter().map(|w| w.0).collect::<Vec<_>>().join(" | ")))), pod("Table", "Works", table(Some(vec!["title", "year"]), t))]));
    }
    // "number of scenes in the Tempest act 3"
    if let Some(m) = re!(r"^(?:the )?number of scenes in (.+?),? act (\d+)$").captures(l) {
        let w = find(&m[1])?;
        let a: usize = m[2].parse().ok()?;
        return answer_many(&[w], "scenes", Some(a), &format!("scenes in act {a}"));
    }
    // counts and attributes: "number of chapters in On the Origin of Species, Walden",
    // "compare the number of scenes in King Lear, King John, King Richard II", "number of acts in Othello",
    // "number of words in the US constitution", "original language of War and Peace, Lolita"
    if let Some(m) = re!(r"^(?:compare )?(?:the )?(?:number of (chapters|acts|scenes|words|books|cantos|verses|surahs)|(original language|language|original publication date|publication date|publication year|author|word count|length)) (?:in|of|for) (.+)$").captures(l) {
        let what = match m.get(1).map(|x| x.as_str()).or(m.get(2).map(|x| x.as_str()))? {
            "chapters" | "books" | "cantos" | "surahs" => "chapters",
            "acts" => "acts",
            "scenes" => "scenes",
            "words" | "word count" | "length" => "words",
            "original language" | "language" => "language",
            "author" => "author",
            _ => "published",
        };
        let ws = split(&m[3])?;
        let label = match what {
            "chapters" => "chapters",
            "acts" => "acts",
            "scenes" => "scenes",
            "words" => "words",
            "language" => "original language",
            "author" => "author",
            _ => "first published",
        };
        return answer_many(&ws, what, None, label);
    }
    // "When was Dune published?", "who wrote Moby Dick"
    if let Some(m) = re!(r"^when (?:was|were) (.+?) (?:first )?(?:published|written|released)$").captures(l) {
        let ws = split(&m[1])?;
        return answer_many(&ws, "published", None, "first published");
    }
    if let Some(m) = re!(r"^who wrote (.+)$").captures(l).or_else(|| re!(r"^(?:the )?author of (.+)$").captures(l)) {
        let ws = split(&m[1])?;
        return answer_many(&ws, "author", None, "author");
    }
    // "Pentateuch" books
    if l == "books of the pentateuch" || l == "books of the torah" {
        return Some(Answer::new("text", vec![primary(pod("Result", "Books", text("Genesis | Exodus | Leviticus | Numbers | Deuteronomy")))]));
    }
    // titles: "Moby Dick", "Hedda Gabler, Othello", "Plato's Republic vs Walden", "Naked Lunch, 1st to Die"
    let ws = split(l)?;
    if ws.len() == 1 {
        // a few titles are also everyday words
        if matches!(l, "dune" | "ghosts" | "emma" | "the tempest" | "tempest" | "walden" | "sonnets" | "1984" | "odyssey" | "iliad" | "republic" | "ulysses" | "analects" | "the constitution") && q.lower.split_whitespace().count() == 1 {
            return None;
        }
        return Some(card(ws[0]));
    }
    let mut head = vec![""];
    head.extend(ws.iter().map(|w| w.title()));
    let rowsf: [(&str, fn(&Work) -> String); 5] = [
        ("author", |w| w.author.clone()),
        ("first published", |w| w.year.clone()),
        ("original language", |w| w.language.clone()),
        ("form", |w| w.form.clone()),
        ("length", |w| w.words.map(|n| format!("about {} words", group(n))).unwrap_or_default()),
    ];
    let t = rowsf.iter().map(|(k, f)| {
        let mut r = vec![text(*k)];
        r.extend(ws.iter().map(|w| text(f(w))));
        r
    }).collect();
    let main = ws.iter().map(|w| format!("{}: {}, {}", w.title(), w.author, w.year)).collect::<Vec<_>>().join(" | ");
    Some(Answer::new("texts", vec![pod("Input", "Input interpretation", text(ws.iter().map(|w| w.title()).collect::<Vec<_>>().join(" vs "))), primary(pod("Result", "Result", text(main))), pod("Table", "Comparison", table(Some(head), t))]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_parses() {
        assert_eq!(DATA.lines().filter(|l| !l.trim().is_empty()).count(), works().len());
    }
}
