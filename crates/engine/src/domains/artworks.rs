//! Famous artworks: artist, date, medium, size and where they hang.
//!
//! "Mona Lisa", "The Kiss by Rodin", "which museum exhibits Pieta by
//! Michelangelo?", "who painted Guernica", "paintings by Vermeer".

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
    /// lower-case titles; the first is the display form source
    names: &'static [&'static str],
    title: &'static str,
    /// answers on its own (without "painting", an artist or a question word)
    bare: bool,
    artist: &'static str,
    date: &'static str,
    kind: &'static str,
    medium: &'static str,
    /// height × width (× depth); "" when unknown or variable
    size: &'static str,
    location: &'static str,
}

macro_rules! w {
    ($n:expr, $t:expr, $b:expr, $a:expr, $d:expr, $k:expr, $m:expr, $s:expr, $l:expr) => {
        Work { names: $n, title: $t, bare: $b, artist: $a, date: $d, kind: $k, medium: $m, size: $s, location: $l }
    };
}

const WORKS: &[Work] = &[
    w!(&["mona lisa", "la gioconda", "la joconde"], "Mona Lisa", true, "Leonardo da Vinci", "c. 1503–1519", "painting", "oil on poplar panel", "77 cm × 53 cm", "Louvre, Paris, France"),
    w!(&["the last supper", "last supper"], "The Last Supper", true, "Leonardo da Vinci", "1495–1498", "mural painting", "tempera and oil on gesso", "460 cm × 880 cm", "Santa Maria delle Grazie, Milan, Italy"),
    w!(&["lady with an ermine", "the lady with an ermine"], "Lady with an Ermine", true, "Leonardo da Vinci", "c. 1489–1491", "painting", "oil on walnut panel", "54 cm × 39 cm", "Czartoryski Museum, Kraków, Poland"),
    w!(&["vitruvian man", "the vitruvian man"], "Vitruvian Man", true, "Leonardo da Vinci", "c. 1490", "drawing", "pen and ink with wash over metalpoint on paper", "34.6 cm × 25.5 cm", "Gallerie dell'Accademia, Venice, Italy"),
    w!(&["pieta", "pietà", "the pieta", "the pietà", "vatican pieta"], "Pietà", true, "Michelangelo", "1498–1499", "sculpture", "Carrara marble", "174 cm × 195 cm", "St. Peter's Basilica, Vatican City"),
    w!(&["david", "statue of david"], "David", false, "Michelangelo", "1501–1504", "sculpture", "marble", "517 cm (with plinth)", "Galleria dell'Accademia, Florence, Italy"),
    w!(&["the creation of adam", "creation of adam"], "The Creation of Adam", true, "Michelangelo", "c. 1508–1512", "fresco", "fresco", "280 cm × 570 cm", "Sistine Chapel, Vatican City"),
    w!(&["sistine chapel ceiling", "the sistine chapel ceiling"], "Sistine Chapel ceiling", true, "Michelangelo", "1508–1512", "fresco cycle", "fresco", "about 40 m × 13 m", "Sistine Chapel, Vatican City"),
    w!(&["the school of athens", "school of athens"], "The School of Athens", true, "Raphael", "1509–1511", "fresco", "fresco", "500 cm × 770 cm", "Apostolic Palace (Raphael Rooms), Vatican City"),
    w!(&["the birth of venus", "birth of venus"], "The Birth of Venus", true, "Sandro Botticelli", "c. 1484–1486", "painting", "tempera on canvas", "172.5 cm × 278.9 cm", "Uffizi Gallery, Florence, Italy"),
    w!(&["primavera"], "Primavera", false, "Sandro Botticelli", "c. 1480", "painting", "tempera on panel", "202 cm × 314 cm", "Uffizi Gallery, Florence, Italy"),
    w!(&["the arnolfini portrait", "arnolfini portrait"], "The Arnolfini Portrait", true, "Jan van Eyck", "1434", "painting", "oil on oak panel", "82.2 cm × 60 cm", "National Gallery, London, United Kingdom"),
    w!(&["the garden of earthly delights", "garden of earthly delights"], "The Garden of Earthly Delights", true, "Hieronymus Bosch", "c. 1490–1510", "triptych", "oil on oak panels", "205.5 cm × 384.9 cm", "Museo del Prado, Madrid, Spain"),
    w!(&["girl with a pearl earring"], "Girl with a Pearl Earring", true, "Johannes Vermeer", "c. 1665", "painting", "oil on canvas", "44.5 cm × 39 cm", "Mauritshuis, The Hague, Netherlands"),
    w!(&["the milkmaid"], "The Milkmaid", false, "Johannes Vermeer", "c. 1657–1658", "painting", "oil on canvas", "45.5 cm × 41 cm", "Rijksmuseum, Amsterdam, Netherlands"),
    w!(&["the night watch", "night watch"], "The Night Watch", true, "Rembrandt van Rijn", "1642", "painting", "oil on canvas", "379.5 cm × 453.5 cm", "Rijksmuseum, Amsterdam, Netherlands"),
    w!(&["las meninas"], "Las Meninas", true, "Diego Velázquez", "1656", "painting", "oil on canvas", "318 cm × 276 cm", "Museo del Prado, Madrid, Spain"),
    w!(&["the third of may 1808", "third of may 1808", "the third of may"], "The Third of May 1808", true, "Francisco Goya", "1814", "painting", "oil on canvas", "268 cm × 347 cm", "Museo del Prado, Madrid, Spain"),
    w!(&["saturn devouring his son"], "Saturn Devouring His Son", true, "Francisco Goya", "1819–1823", "painting", "oil mural transferred to canvas", "143.5 cm × 81.4 cm", "Museo del Prado, Madrid, Spain"),
    w!(&["liberty leading the people"], "Liberty Leading the People", true, "Eugène Delacroix", "1830", "painting", "oil on canvas", "260 cm × 325 cm", "Louvre, Paris, France"),
    w!(&["the raft of the medusa", "raft of the medusa"], "The Raft of the Medusa", true, "Théodore Géricault", "1818–1819", "painting", "oil on canvas", "491 cm × 716 cm", "Louvre, Paris, France"),
    w!(&["wanderer above the sea of fog", "the wanderer above the sea of fog"], "Wanderer above the Sea of Fog", true, "Caspar David Friedrich", "c. 1818", "painting", "oil on canvas", "94.8 cm × 74.8 cm", "Hamburger Kunsthalle, Hamburg, Germany"),
    w!(&["the hay wain", "hay wain"], "The Hay Wain", true, "John Constable", "1821", "painting", "oil on canvas", "130.2 cm × 185.4 cm", "National Gallery, London, United Kingdom"),
    w!(&["the fighting temeraire", "fighting temeraire"], "The Fighting Temeraire", true, "J. M. W. Turner", "1839", "painting", "oil on canvas", "90.7 cm × 121.6 cm", "National Gallery, London, United Kingdom"),
    w!(&["the gleaners"], "The Gleaners", true, "Jean-François Millet", "1857", "painting", "oil on canvas", "83.5 cm × 110 cm", "Musée d'Orsay, Paris, France"),
    w!(&["whistler's mother", "arrangement in grey and black no. 1", "arrangement in grey and black"], "Whistler's Mother (Arrangement in Grey and Black No. 1)", true, "James McNeill Whistler", "1871", "painting", "oil on canvas", "144.3 cm × 162.4 cm", "Musée d'Orsay, Paris, France"),
    w!(&["impression, sunrise", "impression sunrise"], "Impression, Sunrise", true, "Claude Monet", "1872", "painting", "oil on canvas", "48 cm × 63 cm", "Musée Marmottan Monet, Paris, France"),
    w!(&["water lilies"], "Water Lilies (series)", false, "Claude Monet", "1897–1926", "series of about 250 paintings", "oil on canvas", "", "many museums; the large panels are in the Musée de l'Orangerie, Paris, France"),
    w!(&["bal du moulin de la galette", "dance at le moulin de la galette"], "Bal du moulin de la Galette", true, "Pierre-Auguste Renoir", "1876", "painting", "oil on canvas", "131 cm × 175 cm", "Musée d'Orsay, Paris, France"),
    w!(&["luncheon of the boating party"], "Luncheon of the Boating Party", true, "Pierre-Auguste Renoir", "1880–1881", "painting", "oil on canvas", "130.2 cm × 175.6 cm", "Phillips Collection, Washington, D.C., United States"),
    w!(&["a sunday afternoon on the island of la grande jatte", "a sunday on la grande jatte", "sunday afternoon on the island of la grande jatte", "la grande jatte"], "A Sunday Afternoon on the Island of La Grande Jatte", true, "Georges Seurat", "1884–1886", "painting", "oil on canvas", "207.6 cm × 308 cm", "Art Institute of Chicago, Chicago, United States"),
    w!(&["the starry night", "starry night"], "The Starry Night", true, "Vincent van Gogh", "1889", "painting", "oil on canvas", "73.7 cm × 92.1 cm", "Museum of Modern Art, New York City, United States"),
    w!(&["cafe terrace at night", "café terrace at night"], "Café Terrace at Night", true, "Vincent van Gogh", "1888", "painting", "oil on canvas", "80.7 cm × 65.3 cm", "Kröller-Müller Museum, Otterlo, Netherlands"),
    w!(&["the potato eaters", "potato eaters"], "The Potato Eaters", true, "Vincent van Gogh", "1885", "painting", "oil on canvas", "82 cm × 114 cm", "Van Gogh Museum, Amsterdam, Netherlands"),
    w!(&["the scream"], "The Scream", true, "Edvard Munch", "1893", "painting", "tempera and crayon on cardboard", "91 cm × 73.5 cm", "National Museum, Oslo, Norway (other versions in the Munch Museum)"),
    w!(&["the kiss (klimt)", "the kiss", "kiss"], "The Kiss (Klimt)", false, "Gustav Klimt", "1907–1908", "painting", "oil and gold leaf on canvas", "180 cm × 180 cm", "Österreichische Galerie Belvedere, Vienna, Austria"),
    w!(&["the kiss (rodin)", "the kiss", "kiss"], "The Kiss (Rodin)", false, "Auguste Rodin", "1882 (marble carved 1888–1898)", "sculpture", "marble", "181.5 cm × 112.5 cm × 117 cm", "Musée Rodin, Paris, France"),
    w!(&["the thinker", "le penseur"], "The Thinker", true, "Auguste Rodin", "1880 (enlarged 1903)", "sculpture", "bronze", "186 cm tall", "Musée Rodin, Paris, France (and more than 20 other casts)"),
    w!(&["the burghers of calais", "burghers of calais"], "The Burghers of Calais", true, "Auguste Rodin", "1884–1889", "sculpture", "bronze", "", "Calais, France (and about a dozen other casts)"),
    w!(&["les demoiselles d'avignon", "demoiselles d'avignon"], "Les Demoiselles d'Avignon", true, "Pablo Picasso", "1907", "painting", "oil on canvas", "243.9 cm × 233.7 cm", "Museum of Modern Art, New York City, United States"),
    w!(&["guernica"], "Guernica", false, "Pablo Picasso", "1937", "painting", "oil on canvas", "349.3 cm × 776.6 cm", "Museo Reina Sofía, Madrid, Spain"),
    w!(&["the dance", "dance", "la danse"], "Dance (La Danse)", false, "Henri Matisse", "1910", "painting", "oil on canvas", "260 cm × 391 cm", "Hermitage Museum, Saint Petersburg, Russia"),
    w!(&["the persistence of memory", "persistence of memory"], "The Persistence of Memory", true, "Salvador Dalí", "1931", "painting", "oil on canvas", "24 cm × 33 cm", "Museum of Modern Art, New York City, United States"),
    w!(&["the treachery of images", "treachery of images", "ceci n'est pas une pipe"], "The Treachery of Images", true, "René Magritte", "1929", "painting", "oil on canvas", "60.3 cm × 81.1 cm", "Los Angeles County Museum of Art, Los Angeles, United States"),
    w!(&["the son of man", "son of man"], "The Son of Man", false, "René Magritte", "1964", "painting", "oil on canvas", "116 cm × 89 cm", "private collection"),
    w!(&["composition with red, blue and yellow", "composition with red blue and yellow"], "Composition with Red, Blue and Yellow", true, "Piet Mondrian", "1930", "painting", "oil on canvas", "46 cm × 46 cm", "Kunsthaus Zürich, Zürich, Switzerland"),
    w!(&["american gothic"], "American Gothic", true, "Grant Wood", "1930", "painting", "oil on beaverboard", "78 cm × 65.3 cm", "Art Institute of Chicago, Chicago, United States"),
    w!(&["nighthawks"], "Nighthawks", true, "Edward Hopper", "1942", "painting", "oil on canvas", "84.1 cm × 152.4 cm", "Art Institute of Chicago, Chicago, United States"),
    w!(&["christina's world", "christinas world"], "Christina's World", true, "Andrew Wyeth", "1948", "painting", "tempera on panel", "81.9 cm × 121.3 cm", "Museum of Modern Art, New York City, United States"),
    w!(&["washington crossing the delaware"], "Washington Crossing the Delaware", true, "Emanuel Leutze", "1851", "painting", "oil on canvas", "378.5 cm × 647.7 cm", "Metropolitan Museum of Art, New York City, United States"),
    w!(&["campbell's soup cans", "campbells soup cans", "campbell's soup"], "Campbell's Soup Cans", true, "Andy Warhol", "1962", "32 paintings", "synthetic polymer paint on canvas", "51 cm × 41 cm each", "Museum of Modern Art, New York City, United States"),
    w!(&["the great wave off kanagawa", "great wave off kanagawa", "the great wave"], "The Great Wave off Kanagawa", true, "Katsushika Hokusai", "c. 1831", "woodblock print", "ink and colour on paper", "25.7 cm × 37.9 cm", "many impressions; e.g. Metropolitan Museum of Art, British Museum"),
    w!(&["venus de milo", "aphrodite of milos"], "Venus de Milo", true, "attributed to Alexandros of Antioch", "c. 150–125 BC", "sculpture", "Parian marble", "204 cm tall", "Louvre, Paris, France"),
    w!(&["winged victory of samothrace", "nike of samothrace", "winged victory"], "Winged Victory of Samothrace", true, "unknown (Hellenistic)", "c. 200–190 BC", "sculpture", "Parian marble", "244 cm tall", "Louvre, Paris, France"),
];

fn surname(artist: &str) -> String {
    let a = artist.to_lowercase();
    let a = a.trim_start_matches("attributed to ");
    match a {
        "leonardo da vinci" => "leonardo|da vinci|leonardo da vinci".into(),
        "rembrandt van rijn" => "rembrandt".into(),
        "vincent van gogh" => "van gogh|gogh|vincent van gogh".into(),
        "jan van eyck" => "van eyck|jan van eyck".into(),
        "j. m. w. turner" => "turner|j. m. w. turner|jmw turner".into(),
        "caspar david friedrich" => "friedrich|caspar david friedrich".into(),
        _ => {
            let last = a.split(' ').last().unwrap_or(a).to_string();
            format!("{last}|{a}")
        }
    }
}

fn by_artist(w: &Work, who: &str) -> bool {
    let who = who.trim().trim_start_matches("the ");
    let n = |s: &str| s.replace(['é', 'è'], "e").replace('á', "a").replace('í', "i");
    surname(w.artist).split('|').any(|s| n(s) == n(who))
}

fn norm(s: &str) -> String {
    s.trim().trim_matches('"').replace('’', "'").replace("pietà", "pieta").replace("café", "cafe")
}

fn find(title: &str, artist: Option<&str>, explicit: bool) -> Vec<&'static Work> {
    let t = norm(title);
    let hits: Vec<&Work> = WORKS.iter().filter(|w| w.names.iter().any(|n| norm(n) == t)).collect();
    let hits: Vec<&Work> = match artist {
        Some(a) => hits.into_iter().filter(|w| by_artist(w, a)).collect(),
        None => hits,
    };
    if !explicit && artist.is_none() && !hits.iter().all(|w| w.bare) {
        return vec![];
    }
    hits
}

fn summary(w: &Work) -> Answer {
    let mut rows = vec![
        vec![text("artist"), text(w.artist)],
        vec![text("date"), text(w.date)],
        vec![text("type"), text(w.kind)],
        vec![text("medium"), text(w.medium)],
    ];
    if !w.size.is_empty() {
        rows.push(vec![text("dimensions"), text(w.size)]);
    }
    rows.push(vec![text("location"), text(w.location)]);
    Answer::new(w.title, vec![
        pod("Input", "Input interpretation", text(format!("{} (artwork)", w.title))),
        primary(pod("Result", "Basic information", text(format!("{} | {} | {}", w.artist, w.date, w.location)))),
        pod("Table", "Properties", table(None, rows)),
    ])
}

fn property(w: &Work, prop: &str) -> Answer {
    let (title, val) = match prop {
        "location" => ("Location", w.location.to_string()),
        "artist" => ("Artist", w.artist.to_string()),
        "date" => ("Date", w.date.to_string()),
        "size" => ("Dimensions", if w.size.is_empty() { "not a single fixed size".to_string() } else { w.size.to_string() }),
        _ => ("Medium", w.medium.to_string()),
    };
    let mut a = summary(w);
    a.pods[1] = primary(pod("Result", title, text(val)));
    a
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim().replace('’', "'");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();

    // "paintings by Vermeer", "works by Rodin", "famous paintings by van gogh"
    if let Some(m) = re!(r"^(?:famous |notable )?(?:paintings|works|artworks|sculptures|works of art) (?:by|of) (.+)$").captures(&l) {
        let ws: Vec<&Work> = WORKS.iter().filter(|w| by_artist(w, &m[1])).collect();
        if ws.is_empty() {
            return None;
        }
        let rows = ws.iter().map(|w| vec![text(w.title), text(w.date), text(w.kind), text(w.location)]).collect();
        return Some(Answer::new(format!("works by {}", ws[0].artist), vec![
            pod("Input", "Input interpretation", text(format!("notable works by {}", ws[0].artist))),
            primary(pod("Result", "Notable works", text(ws.iter().map(|w| w.title).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Details", table(Some(vec!["work", "date", "type", "location"]), rows)),
        ]));
    }

    // property questions
    let props: &[(&Regex, &str)] = &[
        (re!(r"^(?:where is|where can i see|where (?:is|are) (?:the )?(?:painting|sculpture|statue))\s+(?:the painting |the sculpture |the statue )?(.+?)(?: (?:located|displayed|kept|exhibited|on display))?$"), "location"),
        (re!(r"^(?:which|what) (?:museum|gallery) (?:exhibits|has|holds|houses|displays|owns) (?:the )?(.+)$"), "location"),
        (re!(r"^(?:location|museum) of (?:the )?(.+)$"), "location"),
        (re!(r"^who (?:painted|sculpted|made|created|carved|drew) (.+)$"), "artist"),
        (re!(r"^(?:artist|painter|sculptor) of (.+)$"), "artist"),
        (re!(r"^when (?:was|were) (.+?) (?:painted|sculpted|made|created|carved|completed)$"), "date"),
        (re!(r"^(?:how big is|how large is|size of|dimensions of) (?:the )?(.+)$"), "size"),
    ];
    for (r, p) in props {
        if let Some(m) = r.captures(&l) {
            let hits = lookup(&m[1], true);
            // "where is david": a city or person is likelier unless the title is distinctive
            let hits: Vec<&Work> = if *p == "location" && !m[1].contains(" by ") { hits.into_iter().filter(|w| w.bare).collect() } else { hits };
            if hits.len() == 1 {
                return Some(property(hits[0], p));
            }
            if hits.len() > 1 && *p == "artist" {
                let rows = hits.iter().map(|w| vec![text(w.title), text(w.artist), text(w.date)]).collect();
                return Some(Answer::new("artworks", vec![
                    pod("Input", "Input interpretation", text(format!("{} (artworks)", hits[0].title))),
                    primary(pod("Result", "Artists", text(hits.iter().map(|w| format!("{}: {}", w.title, w.artist)).collect::<Vec<_>>().join(" | ")))),
                    pod("Table", "Works with this title", table(Some(vec!["work", "artist", "date"]), rows)),
                ]));
            }
        }
    }

    // "The Kiss by Rodin", "Mona Lisa", "Guernica painting"
    let (s, explicit) = match re!(r"^(.+?) (?:painting|sculpture|statue|artwork|fresco)$").captures(&l) {
        Some(m) => (m[1].to_string(), true),
        None => (l.clone(), false),
    };
    // bare titles ("Girl with a Pearl Earring") are left to the general titles handler
    if !explicit && !s.contains(" by ") && !s.contains("'s ") {
        return None;
    }
    let hits = lookup(&s, explicit);
    match hits.len() {
        1 => Some(summary(hits[0])),
        n if n > 1 && explicit => Some(summary(hits[0])),
        _ => None,
    }
}

/// the whole phrase as a title first ("whistler's mother"), then with an artist split off
fn lookup(s: &str, explicit: bool) -> Vec<&'static Work> {
    let s = s.trim().trim_start_matches("the painting ").trim_start_matches("the sculpture ").trim_start_matches("the statue ");
    let whole = find(s, None, explicit);
    if !whole.is_empty() {
        return whole;
    }
    let (title, artist) = split_artist(s);
    match artist {
        Some(a) => find(&title, Some(&a), explicit),
        None => vec![],
    }
}

/// "pieta by michelangelo" → ("pieta", Some("michelangelo")); also "rodin's the kiss"
fn split_artist(s: &str) -> (String, Option<String>) {
    let s = s.trim().trim_start_matches("the painting ").trim_start_matches("the sculpture ");
    if let Some(m) = re!(r"^(.+?) (?:by|from) (.+)$").captures(s) {
        return (m[1].to_string(), Some(m[2].to_string()));
    }
    if let Some(m) = re!(r"^([a-zé ]+?)'s (.+)$").captures(s) {
        let cand = m[1].to_string();
        if WORKS.iter().any(|w| by_artist(w, &cand)) {
            return (m[2].to_string(), Some(cand));
        }
    }
    (s.to_string(), None)
}
