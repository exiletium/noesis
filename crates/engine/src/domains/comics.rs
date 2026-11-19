//! Comics, manga and graphic novels, periodicals and broadcast networks:
//! creators, publishers and dates.
//!
//! "publisher Sandman comic", "creator of Absolute Boyfriend", "Bleach comic
//! vs. X-Men comic", "Harper's Magazine, The Atlantic", "ABC, NBC, CBS, FOX".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

struct Item {
    title: &'static str,
    aliases: &'static [&'static str],
    kind: &'static str,
    creators: &'static str,
    publisher: &'static str,
    dates: &'static str,
    note: &'static str,
}

const COMICS: &[Item] = &[
    Item { title: "The Sandman", aliases: &["sandman", "the sandman"], kind: "comic book series", creators: "Neil Gaiman (writer), with Sam Kieth, Mike Dringenberg and others", publisher: "DC Comics (Vertigo imprint from 1993)", dates: "1989–1996 (75 issues)", note: "follows Dream of the Endless" },
    Item { title: "Hellboy: Seed of Destruction", aliases: &["hellboy: seed of destruction", "hellboy seed of destruction", "seed of destruction"], kind: "comic book miniseries", creators: "Mike Mignola (story and art), John Byrne (script)", publisher: "Dark Horse Comics", dates: "1994 (4 issues)", note: "the first Hellboy story arc" },
    Item { title: "Preacher", aliases: &["preacher"], kind: "comic book series", creators: "Garth Ennis (writer), Steve Dillon (artist)", publisher: "DC Comics (Vertigo)", dates: "1995–2000 (66 issues)", note: "" },
    Item { title: "30 Days of Night", aliases: &["30 days of night"], kind: "comic book miniseries", creators: "Steve Niles (writer), Ben Templesmith (artist)", publisher: "IDW Publishing", dates: "2002 (3 issues)", note: "vampires in Barrow, Alaska, during a month-long polar night; filmed in 2007" },
    Item { title: "Absolute Boyfriend", aliases: &["absolute boyfriend", "zettai kareshi"], kind: "manga", creators: "Yuu Watase", publisher: "Shogakukan (Shōjo Comic); Viz Media in English", dates: "2003–2005 (6 volumes)", note: "" },
    Item { title: "Sailor Moon", aliases: &["sailor moon", "pretty guardian sailor moon", "bishōjo senshi sērā mūn"], kind: "manga", creators: "Naoko Takeuchi", publisher: "Kodansha (Nakayoshi)", dates: "1991–1997 (18 volumes)", note: "" },
    Item { title: "Barefoot Gen", aliases: &["barefoot gen", "hadashi no gen"], kind: "manga", creators: "Keiji Nakazawa", publisher: "Shueisha (Weekly Shōnen Jump) and later other magazines", dates: "1973–1987 (10 volumes)", note: "semi-autobiographical story of a boy who survives the atomic bombing of Hiroshima" },
    Item { title: "300", aliases: &["300", "300 graphic novel"], kind: "comic book miniseries (graphic novel)", creators: "Frank Miller (writer and artist), Lynn Varley (colorist)", publisher: "Dark Horse Comics", dates: "1998 (5 issues)", note: "the Battle of Thermopylae; filmed in 2006" },
    Item { title: "303", aliases: &["303", "303 comic books"], kind: "comic book miniseries", creators: "Garth Ennis (writer), Jacen Burrows (artist)", publisher: "Avatar Press", dates: "2004–2005 (6 issues)", note: "named after the .303 Lee–Enfield rifle" },
    Item { title: "Bleach", aliases: &["bleach"], kind: "manga", creators: "Tite Kubo", publisher: "Shueisha (Weekly Shōnen Jump)", dates: "2001–2016 (74 volumes)", note: "" },
    Item { title: "X-Men", aliases: &["x-men", "x men", "the x-men", "uncanny x-men"], kind: "comic book series", creators: "Stan Lee (writer), Jack Kirby (artist)", publisher: "Marvel Comics", dates: "1963–", note: "the first issue appeared in September 1963" },
    Item { title: "Akira", aliases: &["akira"], kind: "manga", creators: "Katsuhiro Otomo", publisher: "Kodansha (Young Magazine)", dates: "1982–1990 (6 volumes)", note: "Otomo directed the 1988 anime film" },
    Item { title: "Cable & Deadpool", aliases: &["cable and deadpool", "cable & deadpool"], kind: "comic book series", creators: "Fabian Nicieza (writer), Mark Brooks, Patrick Zircher and others (artists)", publisher: "Marvel Comics", dates: "2004–2008 (50 issues)", note: "" },
    Item { title: "Alias", aliases: &["alias"], kind: "comic book series", creators: "Brian Michael Bendis (writer), Michael Gaydos (artist)", publisher: "Marvel Comics (MAX imprint)", dates: "2001–2004 (28 issues)", note: "introduced Jessica Jones" },
    Item { title: "The Pulse", aliases: &["the pulse"], kind: "comic book series", creators: "Brian Michael Bendis (writer), Mark Bagley, Michael Lark and Michael Gaydos (artists)", publisher: "Marvel Comics", dates: "2004–2006 (14 issues)", note: "continued the story of Jessica Jones from Alias" },
    Item { title: "The Batman Adventures: Mad Love", aliases: &["the batman adventures: mad love", "mad love", "batman adventures mad love"], kind: "one-shot comic", creators: "Paul Dini (writer), Bruce Timm (artist)", publisher: "DC Comics", dates: "1994", note: "Harley Quinn's origin; won the 1994 Eisner Award for best single issue" },
    Item { title: "Gotham City Sirens", aliases: &["gotham city sirens"], kind: "comic book series", creators: "Paul Dini (writer), Guillem March (artist)", publisher: "DC Comics", dates: "2009–2011 (26 issues)", note: "Catwoman, Harley Quinn and Poison Ivy" },
    Item { title: "The Walking Dead", aliases: &["the walking dead", "walking dead"], kind: "comic book series", creators: "Robert Kirkman (writer), Tony Moore and Charlie Adlard (artists)", publisher: "Image Comics", dates: "2003–2019 (193 issues)", note: "adapted as the AMC television series (2010–2022)" },
    Item { title: "Box Office Poison", aliases: &["box office poison"], kind: "comic book series (graphic novel)", creators: "Alex Robinson", publisher: "Antarctic Press (serial); Top Shelf Productions (collected edition, 2001)", dates: "1996–2000", note: "" },
    Item { title: "Batman", aliases: &["batman"], kind: "comic book series", creators: "Bob Kane and Bill Finger", publisher: "DC Comics", dates: "1940– (the character debuted in Detective Comics #27, May 1939)", note: "" },
    Item { title: "Saga", aliases: &["saga"], kind: "comic book series", creators: "Brian K. Vaughan (writer), Fiona Staples (artist)", publisher: "Image Comics", dates: "2012–", note: "space-opera fantasy; many Eisner and Hugo awards" },
    Item { title: "Gotham by Gaslight", aliases: &["gotham by gaslight"], kind: "one-shot graphic novel", creators: "Brian Augustyn (writer), Mike Mignola (artist)", publisher: "DC Comics", dates: "1989", note: "a Victorian-era Batman hunts Jack the Ripper; regarded as the first Elseworlds story" },
    Item { title: "52", aliases: &["52"], kind: "weekly comic book series", creators: "Geoff Johns, Grant Morrison, Greg Rucka and Mark Waid (writers), Keith Giffen (breakdowns)", publisher: "DC Comics", dates: "May 2006 – May 2007 (52 issues)", note: "told in real time over one year" },
    Item { title: "Naruto", aliases: &["naruto"], kind: "manga", creators: "Masashi Kishimoto", publisher: "Shueisha (Weekly Shōnen Jump)", dates: "1999–2014 (72 volumes)", note: "" },
    Item { title: "The Dark Tower", aliases: &["the dark tower", "dark tower"], kind: "comic book series", creators: "Stephen King (creative director), Robin Furth and Peter David (writers), Jae Lee and Richard Isanove (artists)", publisher: "Marvel Comics", dates: "2007–2013", note: "adapts and expands Stephen King's novel series" },
    Item { title: "Daredevil/Bullseye: The Target", aliases: &["daredevil/bullseye: the target", "daredevil bullseye the target", "daredevil/bullseye the target"], kind: "comic book miniseries", creators: "Kevin Smith (writer), Glenn Fabry (artist)", publisher: "Marvel Comics (Marvel Knights)", dates: "2003 (only issue 1 of a planned 5 was published)", note: "" },
];

const PERIODICALS: &[Item] = &[
    Item { title: "Sports Illustrated", aliases: &["sports illustrated", "si"], kind: "sports magazine", creators: "Henry Luce (founder)", publisher: "Time Inc. until 2018; the brand is owned by Authentic Brands Group", dates: "first issue August 16, 1954", note: "weekly for most of its history, now monthly; known for its annual swimsuit issue" },
    Item { title: "Idaho Statesman", aliases: &["idaho statesman", "the idaho statesman"], kind: "daily newspaper", creators: "", publisher: "McClatchy", dates: "founded 1864 (as the Idaho Tri-Weekly Statesman)", note: "published in Boise, Idaho" },
    Item { title: "Harper's Magazine", aliases: &["harper's magazine", "harpers magazine", "harper's"], kind: "monthly magazine", creators: "Harper & Brothers (founders)", publisher: "Harper's Magazine Foundation", dates: "founded June 1850", note: "New York; the second-oldest continuously published monthly magazine in the United States" },
    Item { title: "The Atlantic", aliases: &["the atlantic", "atlantic monthly", "the atlantic monthly"], kind: "magazine", creators: "Francis H. Underwood, with Ralph Waldo Emerson, Henry Wadsworth Longfellow and others", publisher: "The Atlantic Monthly Group (majority-owned by Emerson Collective since 2017)", dates: "founded 1857 in Boston", note: "now based in Washington, D.C." },
    Item { title: "Road & Track", aliases: &["road & track", "road and track"], kind: "automotive magazine", creators: "", publisher: "Hearst Magazines", dates: "founded 1947", note: "" },
    Item { title: "The Guardian", aliases: &["the guardian", "guardian", "the guardian periodical", "manchester guardian"], kind: "daily newspaper", creators: "John Edward Taylor (founder)", publisher: "Guardian Media Group (owned by the Scott Trust)", dates: "founded 1821 in Manchester (as The Manchester Guardian)", note: "website: theguardian.com; based in London since 1964" },
    Item { title: "The New York Times", aliases: &["new york times", "the new york times", "nyt"], kind: "daily newspaper", creators: "Henry Jarvis Raymond and George Jones (founders)", publisher: "The New York Times Company", dates: "first issue September 18, 1851", note: "website: nytimes.com" },
    Item { title: "The Wall Street Journal", aliases: &["wall street journal", "the wall street journal", "wsj"], kind: "daily newspaper", creators: "Charles Dow, Edward Jones and Charles Bergstresser (founders)", publisher: "Dow Jones & Company (News Corp)", dates: "first issue July 8, 1889", note: "website: wsj.com" },
];

const NETWORKS: &[Item] = &[
    Item { title: "ABC", aliases: &["abc", "american broadcasting company"], kind: "US broadcast television network", creators: "", publisher: "The Walt Disney Company (since 1996)", dates: "radio network formed 1943 from the NBC Blue Network; television from 1948", note: "" },
    Item { title: "NBC", aliases: &["nbc", "national broadcasting company"], kind: "US broadcast television network", creators: "RCA (founder)", publisher: "NBCUniversal (Comcast)", dates: "radio network from 1926; regular television from 1939", note: "the oldest major US broadcast network" },
    Item { title: "CBS", aliases: &["cbs", "columbia broadcasting system"], kind: "US broadcast television network", creators: "", publisher: "Paramount Skydance", dates: "radio network from 1927; television from 1941", note: "" },
    Item { title: "Fox", aliases: &["fox", "fox broadcasting company", "fox network"], kind: "US broadcast television network", creators: "Rupert Murdoch and Barry Diller", publisher: "Fox Corporation", dates: "launched October 9, 1986", note: "" },
    Item { title: "Telemundo", aliases: &["telemundo"], kind: "US Spanish-language television network", creators: "", publisher: "NBCUniversal (since 2001)", dates: "the name comes from WKAQ-TV in San Juan, Puerto Rico (1954)", note: "" },
    Item { title: "Bloomberg Television", aliases: &["bloomberg television", "bloomberg tv"], kind: "business news television channel", creators: "", publisher: "Bloomberg L.P.", dates: "launched 1994", note: "" },
];

fn norm(s: &str) -> String {
    let s = s.trim().replace('’', "'");
    let s = re!(r"\s+(?:comics?|comic books?|manga|graphic novel|tv network|network|periodical|magazine|newspaper)$").replace(&s, "");
    let s = re!(r"^(?:the )?(?:image(?: of)?|picture of) ").replace(&s, "");
    s.trim().to_string()
}

fn find_in(list: &'static [Item], s: &str) -> Option<&'static Item> {
    let n = norm(s);
    list.iter().find(|i| i.title.to_lowercase() == n || i.aliases.contains(&n.as_str()))
}

fn find(s: &str, comics_only: bool) -> Option<&'static Item> {
    find_in(COMICS, s).or_else(|| if comics_only { None } else { find_in(PERIODICALS, s).or_else(|| find_in(NETWORKS, s)) })
}

fn rows(i: &Item) -> Vec<Vec<Content>> {
    let mut r = vec![vec![text("type"), text(i.kind)]];
    if !i.creators.is_empty() {
        r.push(vec![text("creators"), text(i.creators)]);
    }
    r.push(vec![text(if i.kind.contains("network") || i.kind.contains("channel") { "owner" } else { "publisher" }), text(i.publisher)]);
    r.push(vec![text("dates"), text(i.dates)]);
    if !i.note.is_empty() {
        r.push(vec![text("note"), text(i.note)]);
    }
    r
}

fn card(i: &Item) -> Answer {
    let who = if i.creators.is_empty() { String::new() } else { format!(" by {}", i.creators) };
    Answer::new("publication", vec![
        pod("Input", "Input interpretation", text(format!("{} ({})", i.title, i.kind))),
        primary(pod("Result", "Basic information", text(format!("{}{}; {}; {}", i.kind, who, i.publisher, i.dates)))),
        pod("Properties", "Properties", table(None, rows(i))),
    ])
}

fn split(s: &str, comics_only: bool) -> Option<Vec<&'static Item>> {
    // "Akira comic vs. Cable and Deadpool comic": try without splitting on "and" first
    let a: Vec<&str> = re!(r"\s*(?:,|\bvs\.?(?:\s|$)|\bversus\b)\s*").split(s).filter(|x| !x.trim().is_empty()).collect();
    if a.len() >= 2 {
        if let Some(v) = a.iter().map(|p| find(p, comics_only)).collect::<Option<Vec<_>>>() {
            return Some(v);
        }
    }
    let parts: Vec<&str> = re!(r"\s*(?:,|\bvs\.?(?:\s|$)|\bversus\b|\band\b)\s*").split(s).filter(|x| !x.trim().is_empty()).collect();
    if parts.len() < 2 {
        return None;
    }
    parts.iter().map(|p| find(p, comics_only)).collect()
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim().replace('’', "'");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let l = l.trim_end_matches(" wikipedia summary").trim_end_matches(" wiki summary").trim_end_matches(" wikipedia summaries").to_string();
    let l = l.as_str();
    // "publisher Sandman comic", "who publishes the 30 Days of Night comic", "creator of Absolute Boyfriend", "Road & Track publisher"
    if let Some(m) = re!(r"^(?:who (publishes|published|created|creates|wrote|owns) (?:the )?(.+)|(publisher|creator|creators|author|owner|website)s? (?:of |for )?(?:the )?(.+)|(.+?) (publisher|creator|creators|author|owner|website))$").captures(l) {
        let (p, who) = if let Some(p) = m.get(1) { (p.as_str(), &m[2]) } else if let Some(p) = m.get(3) { (p.as_str(), &m[4]) } else { (&m[6], &m[5]) };
        if let Some(i) = find(who, false) {
            let (label, v) = match p {
                "publishes" | "published" | "publisher" | "owns" | "owner" => (if i.kind.contains("network") || i.kind.contains("channel") { "Owner" } else { "Publisher" }, i.publisher.to_string()),
                "website" => ("Website", i.note.split("website: ").nth(1)?.split(';').next()?.to_string()),
                _ => ("Creators", if i.creators.is_empty() { return None } else { i.creators.to_string() }),
            };
            let mut a = card(i);
            a.pods[0] = pod("Input", "Input interpretation", text(format!("{} | {}", i.title, label.to_lowercase())));
            a.pods[1] = primary(pod("Result", label, text(v)));
            return Some(a);
        }
    }
    // "Bleach comic vs. X-Men comic", "Harper's Magazine, The Atlantic", "ABC, NBC, CBS, FOX"
    let comics_only = l.contains("comic") || l.contains("manga");
    if let Some(v) = split(l, comics_only) {
        let rows = v.iter().map(|i| vec![text(i.title), text(i.kind), text(i.publisher), text(i.dates)]).collect();
        return Some(Answer::new("publications", vec![
            pod("Input", "Input interpretation", text(v.iter().map(|i| i.title).collect::<Vec<_>>().join(" | "))),
            primary(pod("Result", "Comparison", text(v.iter().map(|i| format!("{} ({})", i.title, i.publisher)).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Details", table(Some(vec!["title", "type", "publisher / owner", "dates"]), rows)),
        ]));
    }
    // a single title needs a "comic"/"manga" word or an unambiguous name
    let i = find(l, false)?;
    let tagged = comics_only || l.contains("magazine") || l.contains("newspaper") || l.contains("network") || l.contains("periodical") || l.starts_with("image ");
    let distinctive = matches!(i.title, "Hellboy: Seed of Destruction" | "30 Days of Night" | "Absolute Boyfriend" | "Barefoot Gen" | "Cable & Deadpool" | "The Batman Adventures: Mad Love" | "Gotham City Sirens" | "Box Office Poison" | "Gotham by Gaslight" | "Daredevil/Bullseye: The Target" | "Sports Illustrated" | "Idaho Statesman" | "Harper's Magazine" | "Road & Track" | "Bloomberg Television" | "Telemundo");
    (tagged || distinctive).then(|| card(i))
}
