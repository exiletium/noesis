//! Films, film franchises, filmographies, bands and albums: cast, runtime,
//! ratings, box office and Academy Awards for a set of well-known films.
//!
//! "cast of The Lion King", "director, writer of Schindler's List", "James
//! Bond movies", "Academy Awards won by Meryl Streep", "bands with Eric
//! Clapton and Jimmy Page".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

struct Film {
    title: &'static str,
    aliases: &'static [&'static str],
    year: i32,
    franchise: &'static str,
    directors: &'static str,
    writers: &'static str,
    cast: &'static str,
    /// minutes; 0 when not given
    runtime: u32,
    rating: &'static str,
    /// US$ millions; NaN when not given
    budget: f64,
    gross: f64,
    oscars: &'static str,
    note: &'static str,
}

const N: f64 = f64::NAN;

const FILMS: &[Film] = &[
    Film { title: "The Lion King", aliases: &["lion king"], year: 1994, franchise: "", directors: "Roger Allers; Rob Minkoff", writers: "Irene Mecchi; Jonathan Roberts; Linda Woolverton", cast: "Matthew Broderick (Simba); James Earl Jones (Mufasa); Jeremy Irons (Scar); Jonathan Taylor Thomas (young Simba); Moira Kelly (Nala); Nathan Lane (Timon); Ernie Sabella (Pumbaa); Rowan Atkinson (Zazu); Robert Guillaume (Rafiki); Whoopi Goldberg (Shenzi); Cheech Marin (Banzai); Jim Cummings (Ed); Madge Sinclair (Sarabi); Niketa Calame (young Nala)", runtime: 88, rating: "G", budget: 45.0, gross: 968.0, oscars: "2 wins (Original Score, Original Song \"Can You Feel the Love Tonight\") of 4 nominations", note: "Walt Disney Feature Animation; gross includes re-releases" },
    Film { title: "2001: A Space Odyssey", aliases: &["2001 a space odyssey", "2001: a space odyssey (film)"], year: 1968, franchise: "", directors: "Stanley Kubrick", writers: "Stanley Kubrick; Arthur C. Clarke", cast: "Keir Dullea (Dave Bowman); Gary Lockwood (Frank Poole); William Sylvester (Heywood Floyd); Daniel Richter (Moon-Watcher); Douglas Rain (voice of HAL 9000)", runtime: 142, rating: "G", budget: 10.5, gross: N, oscars: "1 win (Special Visual Effects) of 4 nominations", note: "Kubrick cut about 19 minutes after the premiere (161 minutes) for the 142-minute general release" },
    Film { title: "Schindler's List", aliases: &["schindlers list"], year: 1993, franchise: "", directors: "Steven Spielberg", writers: "Steven Zaillian (screenplay, from Thomas Keneally's novel Schindler's Ark)", cast: "Liam Neeson (Oskar Schindler); Ben Kingsley (Itzhak Stern); Ralph Fiennes (Amon Göth); Caroline Goodall (Emilie Schindler); Embeth Davidtz (Helen Hirsch)", runtime: 195, rating: "R", budget: 22.0, gross: 322.0, oscars: "7 wins (including Best Picture and Best Director) of 12 nominations", note: "" },
    Film { title: "Halloween", aliases: &["halloween (1978)", "halloween 1978"], year: 1978, franchise: "halloween", directors: "John Carpenter", writers: "John Carpenter; Debra Hill", cast: "Donald Pleasence (Dr. Sam Loomis); Jamie Lee Curtis (Laurie Strode); Nick Castle (Michael Myers); P. J. Soles; Nancy Kyes", runtime: 91, rating: "R", budget: 0.325, gross: 70.0, oscars: "", note: "began the Halloween franchise; Carpenter and Hill also wrote Halloween II (1981)" },
    Film { title: "Star Wars: Episode IV – A New Hope", aliases: &["star wars", "a new hope", "star wars 1977", "star wars episode iv"], year: 1977, franchise: "star wars", directors: "George Lucas", writers: "George Lucas", cast: "Mark Hamill; Harrison Ford; Carrie Fisher; Alec Guinness; Peter Cushing", runtime: 121, rating: "PG", budget: 11.0, gross: 775.0, oscars: "6 wins (plus a Special Achievement Award) of 10 nominations", note: "" },
    Film { title: "Star Wars: Episode V – The Empire Strikes Back", aliases: &["the empire strikes back", "empire strikes back"], year: 1980, franchise: "star wars", directors: "Irvin Kershner", writers: "Leigh Brackett; Lawrence Kasdan (story by George Lucas)", cast: "Mark Hamill; Harrison Ford; Carrie Fisher; Billy Dee Williams; Frank Oz (voice of Yoda)", runtime: 124, rating: "PG", budget: N, gross: N, oscars: "", note: "" },
    Film { title: "Star Wars: Episode VI – Return of the Jedi", aliases: &["return of the jedi"], year: 1983, franchise: "star wars", directors: "Richard Marquand", writers: "Lawrence Kasdan; George Lucas", cast: "Mark Hamill; Harrison Ford; Carrie Fisher; Billy Dee Williams", runtime: 0, rating: "PG", budget: N, gross: N, oscars: "", note: "" },
    Film { title: "Star Wars: Episode I – The Phantom Menace", aliases: &["the phantom menace", "phantom menace"], year: 1999, franchise: "star wars", directors: "George Lucas", writers: "George Lucas", cast: "Liam Neeson; Ewan McGregor; Natalie Portman; Jake Lloyd", runtime: 0, rating: "PG", budget: N, gross: N, oscars: "", note: "" },
    Film { title: "Star Wars: Episode II – Attack of the Clones", aliases: &["attack of the clones"], year: 2002, franchise: "star wars", directors: "George Lucas", writers: "George Lucas; Jonathan Hales", cast: "Ewan McGregor; Natalie Portman; Hayden Christensen; Christopher Lee", runtime: 0, rating: "PG", budget: N, gross: N, oscars: "", note: "" },
    Film { title: "Star Wars: Episode III – Revenge of the Sith", aliases: &["revenge of the sith"], year: 2005, franchise: "star wars", directors: "George Lucas", writers: "George Lucas", cast: "Ewan McGregor; Natalie Portman; Hayden Christensen; Ian McDiarmid", runtime: 0, rating: "PG-13", budget: N, gross: N, oscars: "", note: "" },
    Film { title: "Star Wars: Episode VII – The Force Awakens", aliases: &["the force awakens", "force awakens"], year: 2015, franchise: "star wars", directors: "J. J. Abrams", writers: "Lawrence Kasdan; J. J. Abrams; Michael Arndt", cast: "Harrison Ford; Mark Hamill; Carrie Fisher; Adam Driver; Daisy Ridley; John Boyega; Oscar Isaac", runtime: 0, rating: "PG-13", budget: N, gross: N, oscars: "", note: "" },
    Film { title: "Star Wars: Episode VIII – The Last Jedi", aliases: &["the last jedi", "last jedi"], year: 2017, franchise: "star wars", directors: "Rian Johnson", writers: "Rian Johnson", cast: "Mark Hamill; Carrie Fisher; Adam Driver; Daisy Ridley; John Boyega; Oscar Isaac", runtime: 0, rating: "PG-13", budget: N, gross: N, oscars: "", note: "" },
    Film { title: "Star Wars: Episode IX – The Rise of Skywalker", aliases: &["the rise of skywalker", "rise of skywalker"], year: 2019, franchise: "star wars", directors: "J. J. Abrams", writers: "Chris Terrio; J. J. Abrams", cast: "Carrie Fisher; Mark Hamill; Adam Driver; Daisy Ridley; John Boyega; Oscar Isaac", runtime: 0, rating: "PG-13", budget: N, gross: N, oscars: "", note: "Carrie Fisher appears through unused footage shot for The Force Awakens" },
    Film { title: "Rebel Without a Cause", aliases: &[], year: 1955, franchise: "", directors: "Nicholas Ray", writers: "Stewart Stern (screenplay); Irving Shulman (adaptation); story by Nicholas Ray", cast: "James Dean (Jim Stark); Natalie Wood (Judy); Sal Mineo (Plato); Jim Backus; Ann Doran", runtime: 111, rating: "Approved (Production Code, 1955); it predates the MPAA rating system of 1968", budget: 1.5, gross: N, oscars: "3 nominations", note: "released a month after James Dean's death" },
    Film { title: "Midnight Cowboy", aliases: &[], year: 1969, franchise: "", directors: "John Schlesinger", writers: "Waldo Salt (from the novel by James Leo Herlihy)", cast: "Dustin Hoffman (Ratso Rizzo); Jon Voight (Joe Buck); Sylvia Miles; Brenda Vaccaro", runtime: 113, rating: "X at release; re-rated R in 1971", budget: 3.6, gross: 44.8, oscars: "3 wins (Best Picture, Director, Adapted Screenplay) of 7 nominations", note: "the only X-rated film to win Best Picture" },
    Film { title: "How to Train Your Dragon", aliases: &[], year: 2010, franchise: "", directors: "Chris Sanders; Dean DeBlois", writers: "William Davies; Dean DeBlois; Chris Sanders", cast: "Jay Baruchel (Hiccup); Gerard Butler (Stoick); America Ferrera (Astrid); Craig Ferguson; Jonah Hill; Christopher Mintz-Plasse; Kristen Wiig", runtime: 98, rating: "PG", budget: 165.0, gross: 495.0, oscars: "2 nominations", note: "DreamWorks Animation" },
    Film { title: "Scott Pilgrim vs. the World", aliases: &["scott pilgrim", "scott pilgrim vs the world"], year: 2010, franchise: "", directors: "Edgar Wright", writers: "Michael Bacall; Edgar Wright", cast: "Michael Cera; Mary Elizabeth Winstead; Kieran Culkin; Chris Evans; Anna Kendrick; Brie Larson; Aubrey Plaza", runtime: 112, rating: "PG-13", budget: 85.0, gross: 49.0, oscars: "", note: "based on the graphic novels by Bryan Lee O'Malley" },
    Film { title: "Million Dollar Baby", aliases: &[], year: 2004, franchise: "", directors: "Clint Eastwood", writers: "Paul Haggis (from stories by F.X. Toole)", cast: "Clint Eastwood (Frankie Dunn); Hilary Swank (Maggie Fitzgerald); Morgan Freeman (Eddie Dupris)", runtime: 132, rating: "PG-13", budget: 30.0, gross: 217.0, oscars: "4 wins (Best Picture, Director, Actress for Hilary Swank, Supporting Actor for Morgan Freeman) of 7 nominations", note: "" },
    Film { title: "Avatar", aliases: &["avatar (2009)", "avatar 2009"], year: 2009, franchise: "", directors: "James Cameron", writers: "James Cameron", cast: "Sam Worthington; Zoe Saldaña; Sigourney Weaver; Stephen Lang; Michelle Rodriguez", runtime: 162, rating: "PG-13", budget: 237.0, gross: 2923.0, oscars: "3 wins (Art Direction, Cinematography, Visual Effects) of 9 nominations", note: "the highest-grossing film of all time (including re-releases)" },
    Film { title: "Titanic", aliases: &["titanic (1997)", "titanic 1997", "titanic movie", "titanic film"], year: 1997, franchise: "", directors: "James Cameron", writers: "James Cameron", cast: "Leonardo DiCaprio (Jack Dawson); Kate Winslet (Rose DeWitt Bukater); Billy Zane; Kathy Bates; Frances Fisher; Gloria Stuart; Bill Paxton", runtime: 195, rating: "PG-13", budget: 200.0, gross: 2264.0, oscars: "11 wins (including Best Picture) of 14 nominations", note: "about $1.84 billion in its original 1997–1998 run; the rest from re-releases" },
    Film { title: "Cutthroat Island", aliases: &[], year: 1995, franchise: "", directors: "Renny Harlin", writers: "Robert King; Marc Norman", cast: "Geena Davis; Matthew Modine; Frank Langella", runtime: 124, rating: "PG-13", budget: 98.0, gross: 10.0, oscars: "", note: "one of the largest box-office losses in film history" },
    Film { title: "A Few Good Men", aliases: &[], year: 1992, franchise: "", directors: "Rob Reiner", writers: "Aaron Sorkin (from his play)", cast: "Tom Cruise (Lt. Daniel Kaffee); Jack Nicholson (Col. Nathan Jessup); Demi Moore; Kevin Bacon; Kiefer Sutherland; Kevin Pollak; Cuba Gooding Jr.", runtime: 138, rating: "R", budget: 33.0, gross: 243.0, oscars: "4 nominations", note: "" },
    Film { title: "Gaslight", aliases: &["gaslight (1944)", "gaslight 1944"], year: 1944, franchise: "", directors: "George Cukor", writers: "John Van Druten; Walter Reisch; John L. Balderston (from the play by Patrick Hamilton)", cast: "Charles Boyer (Gregory Anton); Ingrid Bergman (Paula Alquist); Joseph Cotten (Brian Cameron); Angela Lansbury (Nancy Oliver); Dame May Whitty", runtime: 114, rating: "Approved (Production Code); it predates the MPAA rating system of 1968", budget: N, gross: N, oscars: "2 wins (Best Actress for Ingrid Bergman, Black-and-White Art Direction) of 7 nominations", note: "the source of the word gaslighting" },
];

/// James Bond films by Eon Productions: (title, year, Bond actor).
const BOND: &[(&str, i32, &str)] = &[
    ("Dr. No", 1962, "Sean Connery"), ("From Russia with Love", 1963, "Sean Connery"), ("Goldfinger", 1964, "Sean Connery"), ("Thunderball", 1965, "Sean Connery"), ("You Only Live Twice", 1967, "Sean Connery"),
    ("On Her Majesty's Secret Service", 1969, "George Lazenby"), ("Diamonds Are Forever", 1971, "Sean Connery"), ("Live and Let Die", 1973, "Roger Moore"), ("The Man with the Golden Gun", 1974, "Roger Moore"), ("The Spy Who Loved Me", 1977, "Roger Moore"),
    ("Moonraker", 1979, "Roger Moore"), ("For Your Eyes Only", 1981, "Roger Moore"), ("Octopussy", 1983, "Roger Moore"), ("A View to a Kill", 1985, "Roger Moore"), ("The Living Daylights", 1987, "Timothy Dalton"),
    ("Licence to Kill", 1989, "Timothy Dalton"), ("GoldenEye", 1995, "Pierce Brosnan"), ("Tomorrow Never Dies", 1997, "Pierce Brosnan"), ("The World Is Not Enough", 1999, "Pierce Brosnan"), ("Die Another Day", 2002, "Pierce Brosnan"),
    ("Casino Royale", 2006, "Daniel Craig"), ("Quantum of Solace", 2008, "Daniel Craig"), ("Skyfall", 2012, "Daniel Craig"), ("Spectre", 2015, "Daniel Craig"), ("No Time to Die", 2021, "Daniel Craig"),
];

/// Directors' feature films: (director, &[(title, year, approximate worldwide gross in US$ millions or NaN)]).
const DIRECTED: &[(&str, &[(&str, i32, f64)])] = &[
    ("spike lee", &[("She's Gotta Have It", 1986, N), ("School Daze", 1988, N), ("Do the Right Thing", 1989, N), ("Mo' Better Blues", 1990, N), ("Jungle Fever", 1991, N), ("Malcolm X", 1992, N), ("Crooklyn", 1994, N), ("Clockers", 1995, N), ("Girl 6", 1996, N), ("Get on the Bus", 1996, N), ("He Got Game", 1998, N), ("Summer of Sam", 1999, N), ("Bamboozled", 2000, N), ("25th Hour", 2002, N), ("She Hate Me", 2004, N), ("Inside Man", 2006, N), ("Miracle at St. Anna", 2008, N), ("Red Hook Summer", 2012, N), ("Oldboy", 2013, N), ("Da Sweet Blood of Jesus", 2014, N), ("Chi-Raq", 2015, N), ("BlacKkKlansman", 2018, N), ("Da 5 Bloods", 2020, N), ("Highest 2 Lowest", 2025, N)]),
    ("wes anderson", &[("Bottle Rocket", 1996, 0.6), ("Rushmore", 1998, 17.0), ("The Royal Tenenbaums", 2001, 71.0), ("The Life Aquatic with Steve Zissou", 2004, 35.0), ("The Darjeeling Limited", 2007, 35.0), ("Fantastic Mr. Fox", 2009, 46.0), ("Moonrise Kingdom", 2012, 68.0), ("The Grand Budapest Hotel", 2014, 173.0), ("Isle of Dogs", 2018, 64.0), ("The French Dispatch", 2021, 46.0), ("Asteroid City", 2023, 54.0)]),
];

/// Best Actress winners by ceremony year: (ceremony year, winner, film).
const BEST_ACTRESS: &[(i32, &str, &str)] = &[
    (1950, "Olivia de Havilland", "The Heiress"), (1951, "Judy Holliday", "Born Yesterday"), (1952, "Vivien Leigh", "A Streetcar Named Desire"), (1953, "Shirley Booth", "Come Back, Little Sheba"),
    (1954, "Audrey Hepburn", "Roman Holiday"), (1955, "Grace Kelly", "The Country Girl"), (1956, "Anna Magnani", "The Rose Tattoo"), (1957, "Ingrid Bergman", "Anastasia"),
    (1958, "Joanne Woodward", "The Three Faces of Eve"), (1959, "Susan Hayward", "I Want to Live!"), (1960, "Simone Signoret", "Room at the Top"), (1961, "Elizabeth Taylor", "BUtterfield 8"),
    (1962, "Sophia Loren", "Two Women"), (1963, "Anne Bancroft", "The Miracle Worker"), (1964, "Patricia Neal", "Hud"), (1965, "Julie Andrews", "Mary Poppins"),
    (1980, "Sally Field", "Norma Rae"), (1981, "Sissy Spacek", "Coal Miner's Daughter"), (1982, "Katharine Hepburn", "On Golden Pond"), (1983, "Meryl Streep", "Sophie's Choice"),
    (1984, "Shirley MacLaine", "Terms of Endearment"), (1985, "Sally Field", "Places in the Heart"), (1986, "Geraldine Page", "The Trip to Bountiful"), (1987, "Marlee Matlin", "Children of a Lesser God"),
    (1988, "Cher", "Moonstruck"), (1989, "Jodie Foster", "The Accused"), (1990, "Jessica Tandy", "Driving Miss Daisy"),
];

/// Top worldwide grosses of 2019 (US$ millions, approximate).
const TOP_2019: &[(&str, f64)] = &[("Avengers: Endgame", 2799.0), ("The Lion King", 1657.0), ("Frozen II", 1450.0), ("Spider-Man: Far From Home", 1132.0), ("Captain Marvel", 1128.0)];

struct Band {
    name: &'static str,
    aliases: &'static [&'static str],
    origin: &'static str,
    years: &'static str,
    genre: &'static str,
    members: &'static str,
    note: &'static str,
}

const BANDS: &[Band] = &[
    Band { name: "Pink Floyd", aliases: &[], origin: "London, England", years: "1965–1995 (reunions 2005, 2014, 2022)", genre: "progressive rock, psychedelic rock", members: "Syd Barrett; Roger Waters; Nick Mason; Richard Wright; David Gilmour", note: "albums include The Dark Side of the Moon (1973), Wish You Were Here (1975), Animals (1977) and The Wall (1979)" },
    Band { name: "ABBA", aliases: &[], origin: "Stockholm, Sweden", years: "1972–1982; 2016–", genre: "pop, Europop, disco", members: "Agnetha Fältskog; Björn Ulvaeus; Benny Andersson; Anni-Frid Lyngstad", note: "won the 1974 Eurovision Song Contest with \"Waterloo\"" },
    Band { name: "Fleetwood Mac", aliases: &[], origin: "London, England", years: "1967–", genre: "blues rock, soft rock, pop rock", members: "Mick Fleetwood; John McVie; Christine McVie; Lindsey Buckingham; Stevie Nicks; Peter Green; Jeremy Spencer; Danny Kirwan; Bob Welch", note: "Rumours (1977) is one of the best-selling albums of all time" },
    Band { name: "Girls' Generation", aliases: &["girls generation", "snsd", "so nyeo si dae"], origin: "Seoul, South Korea", years: "2007–", genre: "K-pop", members: "Taeyeon; Sunny; Tiffany; Hyoyeon; Yuri; Sooyoung; Yoona; Seohyun; Jessica (until 2014)", note: "an SM Entertainment girl group; \"Gee\" (2009) was a breakthrough hit" },
    Band { name: "The Yardbirds", aliases: &["yardbirds"], origin: "London, England", years: "1963–1968; 1992–", genre: "blues rock, psychedelic rock", members: "Keith Relf; Chris Dreja; Jim McCarty; Paul Samwell-Smith; Anthony \"Top\" Topham; Eric Clapton; Jeff Beck; Jimmy Page", note: "launched three famous guitarists: Eric Clapton (1963–1965), Jeff Beck (1965–1966) and Jimmy Page (1966–1968)" },
    Band { name: "Led Zeppelin", aliases: &[], origin: "London, England", years: "1968–1980", genre: "hard rock, heavy metal, blues rock", members: "Jimmy Page; Robert Plant; John Paul Jones; John Bonham", note: "formed by Jimmy Page as the New Yardbirds" },
    Band { name: "Cream", aliases: &[], origin: "London, England", years: "1966–1968; 1993; 2005", genre: "blues rock, psychedelic rock", members: "Eric Clapton; Jack Bruce; Ginger Baker", note: "" },
    Band { name: "Blind Faith", aliases: &[], origin: "London, England", years: "1968–1969", genre: "blues rock", members: "Eric Clapton; Steve Winwood; Ginger Baker; Ric Grech", note: "" },
    Band { name: "Derek and the Dominos", aliases: &[], origin: "London, England", years: "1970–1971", genre: "blues rock", members: "Eric Clapton; Bobby Whitlock; Carl Radle; Jim Gordon; Duane Allman", note: "recorded \"Layla\" (1970)" },
];

struct Album {
    title: &'static str,
    aliases: &'static [&'static str],
    artist: &'static str,
    year: i32,
    note: &'static str,
}

const ALBUMS: &[Album] = &[
    Album { title: "The Beatles (\"the White Album\")", aliases: &["the white album", "white album"], artist: "The Beatles", year: 1968, note: "a double album of 30 songs with a plain white sleeve" },
    Album { title: "Metallica (\"the Black Album\")", aliases: &["the black album", "black album"], artist: "Metallica", year: 1991, note: "the nickname comes from its black cover; Jay-Z's The Black Album (2003) and Prince's The Black Album (recorded 1987) share the name" },
];

fn norm(s: &str) -> String {
    s.to_lowercase().replace(['–', '—'], "-").replace('’', "'").trim().trim_start_matches("the movie ").trim_end_matches(" movie").trim_end_matches(" film").to_string()
}

fn find(s: &str) -> Option<&'static Film> {
    let s = norm(s);
    let bare = s.trim_start_matches("the ");
    FILMS.iter().find(|f| {
        let t = norm(f.title);
        t == s || t.trim_start_matches("the ") == bare || f.aliases.iter().any(|a| *a == s)
    })
}

fn films(s: &str) -> Option<Vec<&'static Film>> {
    let parts: Vec<&str> = re!(r"\s*(?:,|\bvs\.?\s|\bversus\b|\band\b|/)\s*").split(s).filter(|x| !x.trim().is_empty()).collect();
    if parts.is_empty() {
        return None;
    }
    parts.iter().map(|p| find(p)).collect()
}

fn money(m: f64) -> String {
    if m >= 1000.0 {
        format!("${:.2} billion", m / 1000.0)
    } else if m >= 1.0 {
        format!("${} million", if m.fract() == 0.0 { format!("{m:.0}") } else { format!("{m:.1}") })
    } else {
        format!("${:.0} thousand", m * 1000.0)
    }
}

fn label(f: &Film) -> String {
    format!("{} ({})", f.title, f.year)
}

fn rows(f: &Film) -> Vec<Vec<Content>> {
    let mut r = vec![vec![text("release year"), text(f.year.to_string())], vec![text("directors"), text(f.directors.replace("; ", ", "))], vec![text("writers"), text(f.writers.replace("; ", ", "))]];
    r.push(vec![text("starring"), text(f.cast.split("; ").take(5).collect::<Vec<_>>().join(", "))]);
    if f.runtime > 0 {
        r.push(vec![text("runtime"), text(format!("{} minutes ({} h {} min)", f.runtime, f.runtime / 60, f.runtime % 60))]);
    }
    r.push(vec![text("MPAA rating"), text(f.rating)]);
    if !f.budget.is_nan() {
        r.push(vec![text("production budget"), text(money(f.budget))]);
    }
    if !f.gross.is_nan() {
        r.push(vec![text("worldwide box office"), text(money(f.gross))]);
    }
    if !f.oscars.is_empty() {
        r.push(vec![text("Academy Awards"), text(f.oscars)]);
    }
    if !f.note.is_empty() {
        r.push(vec![text("note"), text(f.note)]);
    }
    r
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn props_pod(f: &Film) -> Pod {
    pod("Properties", "Basic movie information", table(None, rows(f)))
}

fn simple(inp: String, title: &str, v: String, extra: Vec<Pod>) -> Answer {
    let mut pods = vec![input(inp), primary(pod("Result", title, text(v)))];
    pods.extend(extra);
    Answer::new("movie", pods)
}

/// "director, writer of Schindler's List", "2001: A Space Odyssey runtime", "MPAA rating How to Train Your Dragon, Scott Pilgrim"
fn film_props(l: &str) -> Option<Answer> {
    const P: &str = r"cast|cast members|actors|starring|runtime|running time|length|director|directors|writer|writers|screenwriter|director, writer|director and writer|writer, director|mpaa rating|rating|budget|production budget|box office|box office receipts|gross|worldwide gross|academy award nominations|oscar nominations|academy awards|oscars|release date|release year";
    static A: OnceLock<Regex> = OnceLock::new();
    static B: OnceLock<Regex> = OnceLock::new();
    let a = A.get_or_init(|| Regex::new(&format!(r"^(?:what is |who is |who are |what are )?(?:the )?({P}) (?:of|for|in) (.+)$")).unwrap());
    let b = B.get_or_init(|| Regex::new(&format!(r"^(.+?) ({P})$")).unwrap());
    let (p, who) = if let Some(m) = a.captures(l) {
        (m[1].to_string(), m[2].to_string())
    } else if let Some(m) = re!(r"^(mpaa rating|rating|academy award nominations|oscar nominations) (.+)$").captures(l) {
        (m[1].to_string(), m[2].to_string())
    } else {
        let m = b.captures(l)?;
        (m[2].to_string(), m[1].to_string())
    };
    let fs = films(&who)?;
    let one = |f: &Film| -> Option<(String, String)> {
        Some(match p.as_str() {
            "cast" | "cast members" | "actors" | "starring" => ("Cast".into(), f.cast.replace("; ", " | ")),
            "runtime" | "running time" | "length" => ("Runtime".into(), (f.runtime > 0).then(|| format!("{} minutes ({} h {} min)", f.runtime, f.runtime / 60, f.runtime % 60))?),
            "director" | "directors" => ("Director".into(), f.directors.replace("; ", ", ")),
            "writer" | "writers" | "screenwriter" => ("Writer".into(), f.writers.replace("; ", ", ")),
            "director, writer" | "director and writer" | "writer, director" => ("Director and writer".into(), format!("director: {} | writer: {}", f.directors.replace("; ", ", "), f.writers.replace("; ", ", "))),
            "mpaa rating" | "rating" => ("MPAA rating".into(), f.rating.into()),
            "budget" | "production budget" => ("Production budget".into(), (!f.budget.is_nan()).then(|| money(f.budget))?),
            "box office" | "box office receipts" | "gross" | "worldwide gross" => ("Worldwide box office".into(), (!f.gross.is_nan()).then(|| money(f.gross))?),
            "academy award nominations" | "oscar nominations" | "academy awards" | "oscars" => ("Academy Awards".into(), (!f.oscars.is_empty()).then(|| f.oscars.to_string())?),
            _ => ("Release year".into(), f.year.to_string()),
        })
    };
    let vals: Option<Vec<(String, String)>> = fs.iter().map(|f| one(f)).collect();
    let vals = vals?;
    let main = if fs.len() == 1 { vals[0].1.clone() } else { fs.iter().zip(&vals).map(|(f, v)| format!("{}: {}", f.title, v.1)).collect::<Vec<_>>().join(" | ") };
    let extra = if fs.len() == 1 { vec![props_pod(fs[0])] } else { vec![] };
    Some(simple(format!("{} | {}", fs.iter().map(|f| label(f)).collect::<Vec<_>>().join(" | "), vals[0].0.to_lowercase()), &vals[0].0, main, extra))
}

/// "Cutthroat Island total box office / production budget", "Titanic box office with inflation"
fn box_office(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(.+?) (?:total )?(?:box office|gross) ?/ ?(?:production )?budget$").captures(l) {
        let f = find(&m[1])?;
        if f.budget.is_nan() || f.gross.is_nan() {
            return None;
        }
        let r = f.gross / f.budget;
        return Some(simple(format!("{} | box office / budget", label(f)), "Result", format!("{:.2}", r), vec![pod("Values", "Values", table(None, vec![vec![text("worldwide box office"), text(money(f.gross))], vec![text("production budget"), text(money(f.budget))]])), props_pod(f)]));
    }
    if let Some(m) = re!(r"^(.+?) (?:box office|gross)(?: receipts)? (?:with|adjusted for|accounting for) inflation$").captures(l) {
        let f = find(&m[1])?;
        let (from, to) = (crate::domains::trades::cpi(f.year)?, crate::domains::trades::cpi(crate::domains::trades::last_cpi_year())?);
        let base = if f.title == "Titanic" { 1843.0 } else { f.gross };
        if base.is_nan() {
            return None;
        }
        let adj = base * to / from;
        return Some(simple(format!("{} | box office adjusted for inflation", label(f)), "Inflation-adjusted box office", format!("≈ {} ({} dollars)", money(adj), crate::domains::trades::last_cpi_year()), vec![pod("Values", "Values", table(None, vec![vec![text(format!("worldwide box office ({} dollars)", f.year)), text(money(base))], vec![text("US CPI ratio"), text(format!("{:.3}", to / from))]])), pod("Note", "Note", text("adjusted with the US consumer price index from the release year; ticket-price inflation differs somewhat")), props_pod(f)]));
    }
    None
}

fn bond_list() -> Answer {
    let rows = BOND.iter().map(|(t, y, a)| vec![text(*t), text(y.to_string()), text(*a)]).collect();
    simple("James Bond movies".into(), "Films", BOND.iter().map(|(t, _, _)| *t).collect::<Vec<_>>().join(" | "), vec![
        pod("Table", "Eon Productions films", table(Some(vec!["title", "year", "James Bond"]), rows)),
        pod("Note", "Note", text("the 25 official Eon Productions films; Casino Royale (1967) and Never Say Never Again (1983) were made outside the series")),
    ])
}

fn franchise(l: &str) -> Option<Answer> {
    if re!(r"^(?:all |the |list of )?(?:james bond|bond|007) (?:movies|films)(?: in order)?$").is_match(l) {
        return Some(bond_list());
    }
    // "Star Wars movies starring Carrie Fisher"
    if let Some(m) = re!(r"^(.+?) (?:movies|films) (?:starring|with|featuring) (.+)$").captures(l) {
        let fr = m[1].trim_start_matches("the ");
        let who = m[2].trim();
        let v: Vec<&Film> = FILMS.iter().filter(|f| f.franchise == fr && f.cast.to_lowercase().contains(who)).collect();
        if !v.is_empty() {
            let mut extra = vec![pod("Table", "Films", table(Some(vec!["title", "year", "director"]), v.iter().map(|f| vec![text(f.title), text(f.year.to_string()), text(f.directors.replace("; ", ", "))]).collect()))];
            if fr == "star wars" && who.contains("fisher") {
                extra.push(pod("Note", "Note", text("Carrie Fisher's likeness as young Leia also appears briefly, digitally recreated, in Rogue One (2016)")));
            }
            return Some(simple(format!("{} films starring {}", cap_words(fr), cap_words(who)), "Films", v.iter().map(|f| format!("{} ({})", f.title, f.year)).collect::<Vec<_>>().join(" | "), extra));
        }
    }
    // "Halloween franchise writer", "Star Wars franchise writer, director"
    if let Some(m) = re!(r"^(?:the )?(star wars|halloween) (?:franchise|series|movies|films) (writers?|directors?|creators?|writer, director|director, writer|writer and director)$").captures(l) {
        let v: Vec<&Film> = FILMS.iter().filter(|f| f.franchise == &m[1]).collect();
        let main = match &m[1] {
            "star wars" => "created by George Lucas, who wrote and directed Star Wars (1977) and the prequel trilogy (1999–2005)",
            _ => "John Carpenter and Debra Hill, who wrote Halloween (1978), directed by Carpenter, and Halloween II (1981)",
        };
        let rows = v.iter().map(|f| vec![text(f.title), text(f.year.to_string()), text(f.directors.replace("; ", ", ")), text(f.writers.replace("; ", ", "))]).collect();
        return Some(simple(format!("{} franchise | {}", cap_words(&m[1]), &m[2]), "Result", main.into(), vec![pod("Table", "Films", table(Some(vec!["film", "year", "director", "writers"]), rows))]));
    }
    None
}

fn cap_words(s: &str) -> String {
    s.split(' ').map(|w| {
        let mut c = w.chars();
        c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
    }).collect::<Vec<_>>().join(" ")
}

fn people(l: &str) -> Option<Answer> {
    // "movies starring Kevin Bacon and Tom Cruise"
    if let Some(m) = re!(r"^(?:movies|films) (?:starring|with) (.+?) and (.+)$").captures(l) {
        let (a, b) = (m[1].trim(), m[2].trim());
        let v: Vec<&Film> = FILMS.iter().filter(|f| {
            let c = f.cast.to_lowercase();
            c.contains(a) && c.contains(b)
        }).collect();
        if v.is_empty() {
            return None;
        }
        return Some(simple(format!("movies starring {} and {}", cap_words(a), cap_words(b)), "Films", v.iter().map(|f| label(f)).collect::<Vec<_>>().join(" | "), vec![props_pod(v[0]), pod("Note", "Note", text("films in Noesis's built-in list"))]));
    }
    // "movies directed by Spike Lee by release date", "top grossing movies by Wes Anderson"
    if let Some(m) = re!(r"^(?:(top grossing|highest grossing|best selling) )?(?:movies|films) (?:directed )?by (.+?)(?: by (release date|box office|gross))?$").captures(l) {
        let who = m[2].trim();
        let (_, list) = DIRECTED.iter().find(|(n, _)| *n == who)?;
        let by_gross = m.get(1).is_some() || m.get(3).is_some_and(|x| x.as_str() != "release date");
        let mut v: Vec<&(&str, i32, f64)> = list.iter().collect();
        if by_gross {
            if v.iter().any(|x| x.2.is_nan()) {
                return None;
            }
            v.sort_by(|a, b| b.2.total_cmp(&a.2));
        }
        let rows = v.iter().map(|(t, y, g)| vec![text(*t), text(y.to_string()), text(if g.is_nan() { String::new() } else { format!("≈ {}", money(*g)) })]).collect();
        let head = if by_gross { vec!["film", "year", "worldwide gross"] } else { vec!["film", "year", ""] };
        return Some(simple(format!("{} | films{}", cap_words(who), if by_gross { " by box office" } else { " by release date" }), "Films", v.iter().map(|(t, y, _)| format!("{t} ({y})")).collect::<Vec<_>>().join(" | "), vec![pod("Table", "Feature films", table(Some(head), rows))]));
    }
    // "Academy Awards won by Meryl Streep"
    if re!(r"^(?:academy awards|oscars) (?:won by|for|of) meryl streep$|^meryl streep (?:academy awards|oscars)$").is_match(l) {
        let rows = vec![
            vec![text("Best Supporting Actress"), text("Kramer vs. Kramer (1979)"), text("1980")],
            vec![text("Best Actress"), text("Sophie's Choice (1982)"), text("1983")],
            vec![text("Best Actress"), text("The Iron Lady (2011)"), text("2012")],
        ];
        return Some(simple("Meryl Streep | Academy Awards won".into(), "Result", "3 wins: Best Supporting Actress for Kramer vs. Kramer; Best Actress for Sophie's Choice and The Iron Lady".into(), vec![pod("Table", "Wins", table(Some(vec!["award", "film", "ceremony"]), rows)), pod("Nominations", "Nominations", text("21 nominations, the most for any actor"))]));
    }
    // "Oscar for best actress 1982"
    if let Some(m) = re!(r"^(?:the )?(?:oscar|academy award) (?:for |winner for )?best actress (?:in )?(\d{4})$|^best actress (?:oscar|academy award)(?: winner)? (?:in )?(\d{4})$").captures(l) {
        let y: i32 = m.get(1).or(m.get(2))?.as_str().parse().ok()?;
        let cer = BEST_ACTRESS.iter().find(|x| x.0 == y)?;
        let mut rows = vec![vec![text(format!("{y} ceremony (films of {})", y - 1)), text(cer.1), text(cer.2)]];
        let main = if let Some(next) = BEST_ACTRESS.iter().find(|x| x.0 == y + 1) {
            rows.push(vec![text(format!("{} ceremony (films of {y})", y + 1)), text(next.1), text(next.2)]);
            format!("{} for {} ({y} ceremony); for films of {y}: {} for {}", cer.1, cer.2, next.1, next.2)
        } else {
            format!("{} for {}", cer.1, cer.2)
        };
        return Some(simple(format!("Academy Award for Best Actress | {y}"), "Result", main, vec![pod("Table", "Winners", table(Some(vec!["", "winner", "film"]), rows))]));
    }
    // "what is the 4th highest grossing movie of 2019"
    if let Some(m) = re!(r"^(?:what (?:is|was) )?(?:the )?(\d+)(?:st|nd|rd|th)? (?:highest|top)[- ]grossing (?:movie|film) (?:of|in) 2019$").captures(l) {
        let k: usize = m[1].parse().ok()?;
        let (t, g) = TOP_2019.get(k.checked_sub(1)?)?;
        let rows = TOP_2019.iter().enumerate().map(|(i, (t, g))| vec![text((i + 1).to_string()), text(*t), text(format!("≈ {}", money(*g)))]).collect();
        return Some(simple(format!("highest-grossing films of 2019 | rank {k}"), "Result", format!("{t} (≈ {} worldwide)", money(*g)), vec![pod("Table", "Top films of 2019 by worldwide gross", table(Some(vec!["rank", "film", "worldwide gross"]), rows))]));
    }
    None
}

fn music(l: &str) -> Option<Answer> {
    let find_band = |s: &str| {
        let s = s.trim();
        BANDS.iter().find(|b| b.name.to_lowercase() == s || b.aliases.contains(&s) || b.name.to_lowercase().trim_start_matches("the ") == s.trim_start_matches("the "))
    };
    // "bands with Eric Clapton and Jimmy Page"
    if let Some(m) = re!(r"^(?:bands|groups) (?:with|featuring|that included) (.+?)(?: and (.+))?$").captures(l) {
        let a = m[1].trim();
        let b = m.get(2).map(|x| x.as_str().trim());
        let v: Vec<&Band> = BANDS.iter().filter(|x| {
            let mm = x.members.to_lowercase();
            mm.contains(a) && b.is_none_or(|b| mm.contains(b))
        }).collect();
        if v.is_empty() {
            return None;
        }
        let rows = v.iter().map(|x| vec![text(x.name), text(x.years), text(x.members.replace("; ", ", "))]).collect();
        return Some(simple(format!("bands with {}{}", cap_words(a), b.map(|b| format!(" and {}", cap_words(b))).unwrap_or_default()), "Bands", v.iter().map(|x| x.name).collect::<Vec<_>>().join(" | "), vec![pod("Table", "Bands", table(Some(vec!["band", "active", "members"]), rows))]));
    }
    // "the white album vs the black album"
    let parts: Vec<&str> = re!(r"\s*(?:,|\bvs\.?\s|\bversus\b|\band\b)\s*").split(l).filter(|x| !x.trim().is_empty()).collect();
    if parts.is_empty() {
        return None;
    }
    let albums: Option<Vec<&Album>> = parts.iter().map(|p| ALBUMS.iter().find(|a| a.aliases.contains(&p.trim()))).collect();
    if let Some(albums) = albums {
        let rows = albums.iter().map(|a| vec![text(a.title), text(a.artist), text(a.year.to_string()), text(a.note)]).collect();
        return Some(simple(albums.iter().map(|a| a.title).collect::<Vec<_>>().join(" vs "), "Albums", albums.iter().map(|a| format!("{} by {} ({})", a.title, a.artist, a.year)).collect::<Vec<_>>().join(" | "), vec![pod("Table", "Albums", table(Some(vec!["album", "artist", "year", "note"]), rows))]));
    }
    // "pink floyd", "ABBA, Fleetwood Mac, Girls Generation"
    let bands: Option<Vec<&Band>> = parts.iter().map(|p| find_band(p)).collect();
    let bands = bands?;
    if bands.len() == 1 && matches!(l, "cream" | "blind faith") {
        return None;
    }
    if bands.len() == 1 {
        let b = bands[0];
        return Some(simple(format!("{} (music act)", b.name), "Basic information", format!("{} band from {}, active {}", b.genre, b.origin, b.years), vec![pod("Table", "Details", table(None, vec![vec![text("origin"), text(b.origin)], vec![text("years active"), text(b.years)], vec![text("genres"), text(b.genre)], vec![text("members"), text(b.members.replace("; ", ", "))], vec![text("note"), text(b.note)]]))]));
    }
    let rows = bands.iter().map(|b| vec![text(b.name), text(b.origin), text(b.years), text(b.genre)]).collect();
    Some(simple(bands.iter().map(|b| b.name).collect::<Vec<_>>().join(" | "), "Music acts", bands.iter().map(|b| format!("{} ({})", b.name, b.origin)).collect::<Vec<_>>().join(" | "), vec![pod("Table", "Comparison", table(Some(vec!["act", "origin", "active", "genres"]), rows))]))
}

struct Show {
    title: &'static str,
    aliases: &'static [&'static str],
    network: &'static str,
    years: &'static str,
    seasons: u32,
    episodes: u32,
    note: &'static str,
    /// (season, episodes, first aired, last aired)
    season_rows: &'static [(u32, u32, &'static str, &'static str)],
}

const SHOWS: &[Show] = &[
    Show { title: "Game of Thrones", aliases: &["got"], network: "HBO", years: "2011–2019", seasons: 8, episodes: 73, note: "based on George R. R. Martin's A Song of Ice and Fire; created by David Benioff and D. B. Weiss", season_rows: &[(1, 10, "April 17, 2011", "June 19, 2011"), (8, 6, "April 14, 2019", "May 19, 2019")] },
    Show { title: "The Walking Dead", aliases: &["walking dead"], network: "AMC", years: "2010–2022", seasons: 11, episodes: 177, note: "developed by Frank Darabont from the comic book series by Robert Kirkman, Tony Moore and Charlie Adlard", season_rows: &[(1, 6, "October 31, 2010", "December 5, 2010"), (2, 13, "October 16, 2011", "March 18, 2012"), (3, 16, "October 14, 2012", "March 31, 2013")] },
    Show { title: "M*A*S*H", aliases: &["mash", "m*a*s*h", "m.a.s.h."], network: "CBS", years: "1972–1983", seasons: 11, episodes: 256, note: "the finale, \"Goodbye, Farewell and Amen\", aired February 28, 1983 and was watched by about 106 million Americans, long the most-watched US television broadcast", season_rows: &[] },
    Show { title: "The Twilight Zone", aliases: &["twilight zone", "the twilight zone (1959)"], network: "CBS", years: "1959–1964", seasons: 5, episodes: 156, note: "created and hosted by Rod Serling; later revivals ran 1985–1989, 2002–2003 and 2019–2020", season_rows: &[] },
    Show { title: "The Flintstones", aliases: &["flintstones", "the flintstones"], network: "ABC", years: "1960–1966", seasons: 6, episodes: 166, note: "Hanna-Barbera; the first animated series in prime time", season_rows: &[] },
    Show { title: "The Simpsons", aliases: &["simpsons", "the simpsons"], network: "Fox", years: "1989–", seasons: 36, episodes: 790, note: "created by Matt Groening; the longest-running American scripted prime-time series (counts as of the 2024–25 season)", season_rows: &[] },
];

struct Game {
    title: &'static str,
    aliases: &'static [&'static str],
    developer: &'static str,
    publisher: &'static str,
    released: &'static str,
    platforms: &'static str,
    note: &'static str,
}

const GAMES: &[Game] = &[
    Game { title: "Pokémon Red and Blue", aliases: &["pokemon red", "pokémon red", "pokemon blue", "pokemon red and blue"], developer: "Game Freak", publisher: "Nintendo", released: "February 27, 1996 (Japan, as Red and Green); September 28, 1998 (North America)", platforms: "Game Boy", note: "the first Pokémon games, with 151 Pokémon" },
    Game { title: "Need for Speed: Underground", aliases: &["need for speed underground", "nfs underground"], developer: "EA Black Box", publisher: "Electronic Arts", released: "November 17, 2003", platforms: "PlayStation 2, Xbox, GameCube, Microsoft Windows, Game Boy Advance", note: "street racing with car tuning" },
    Game { title: "NBA Jam", aliases: &["nba jam"], developer: "Midway", publisher: "Midway (arcade); Acclaim (home versions)", released: "1993 (arcade)", platforms: "arcade, Super NES, Sega Genesis, Sega CD, Game Gear, Game Boy, Atari Jaguar", note: "two-on-two basketball; \"He's on fire!\"" },
    Game { title: "Zelda II: The Adventure of Link", aliases: &["zelda 2", "zelda ii", "the adventure of link", "zelda ii: the adventure of link"], developer: "Nintendo R&D4", publisher: "Nintendo", released: "January 14, 1987 (Japan); December 1988 (North America)", platforms: "Famicom Disk System, NES", note: "side-scrolling action role-playing sequel to The Legend of Zelda" },
    Game { title: "Call of Duty 4: Modern Warfare", aliases: &["call of duty 4", "cod 4", "cod4", "call of duty 4 modern warfare"], developer: "Infinity Ward", publisher: "Activision", released: "November 5, 2007 (North America, PlayStation 3, Xbox 360 and Windows); November 9, 2007 (Europe)", platforms: "PlayStation 3, Xbox 360, Microsoft Windows, Mac OS X, Nintendo DS (separate version)", note: "" },
];

/// Developers' games: (developer, &[(title, year, platforms)]).
const DEVELOPED: &[(&str, &[(&str, i32, &str)])] = &[
    ("infinity ward", &[("Call of Duty", 2003, "Windows"), ("Call of Duty 2", 2005, "Windows, Xbox 360"), ("Call of Duty 4: Modern Warfare", 2007, "PS3, Xbox 360, Windows"), ("Call of Duty: Modern Warfare 2", 2009, "PS3, Xbox 360, Windows"), ("Call of Duty: Modern Warfare 3", 2011, "PS3, Xbox 360, Windows (with Sledgehammer Games)"), ("Call of Duty: Ghosts", 2013, "PS3, PS4, Xbox 360, Xbox One, Wii U, Windows"), ("Call of Duty: Infinite Warfare", 2016, "PS4, Xbox One, Windows"), ("Call of Duty: Modern Warfare", 2019, "PS4, Xbox One, Windows"), ("Call of Duty: Modern Warfare II", 2022, "PS4, PS5, Xbox One, Xbox Series X/S, Windows")]),
    ("naughty dog", &[("Crash Bandicoot", 1996, "PS1"), ("Jak and Daxter: The Precursor Legacy", 2001, "PS2"), ("Uncharted: Drake's Fortune", 2007, "PS3"), ("Uncharted 2: Among Thieves", 2009, "PS3"), ("Uncharted 3: Drake's Deception", 2011, "PS3"), ("The Last of Us", 2013, "PS3"), ("Uncharted 4: A Thief's End", 2016, "PS4"), ("The Last of Us Part II", 2020, "PS4")]),
];

fn tv_and_games(l: &str) -> Option<Answer> {
    let find_show = |s: &str| {
        let s = s.trim().trim_end_matches(" tv show").trim_end_matches(" tv series");
        SHOWS.iter().find(|x| x.title.to_lowercase() == s || x.aliases.contains(&s))
    };
    // "Game of Thrones season 1"
    if let Some(m) = re!(r"^(.+?) season (\d+)$").captures(l) {
        let sh = find_show(&m[1])?;
        let n: u32 = m[2].parse().ok()?;
        let (_, eps, a, b) = sh.season_rows.iter().find(|r| r.0 == n)?;
        return Some(simple(format!("{} season {n}", sh.title), "Season", format!("{eps} episodes, aired {a} – {b} on {}", sh.network), vec![pod("Show", "Series", table(None, vec![vec![text("network"), text(sh.network)], vec![text("aired"), text(sh.years)], vec![text("seasons"), text(sh.seasons.to_string())], vec![text("episodes"), text(sh.episodes.to_string())]]))]));
    }
    // "when was the finale of mash"
    if let Some(m) = re!(r"^when (?:was|did) (?:the )?(?:series )?finale (?:of )?(.+?)(?: air)?$|^(.+?) (?:series )?finale(?: date)?$").captures(l) {
        let sh = find_show(m.get(1).or(m.get(2))?.as_str())?;
        if sh.title == "M*A*S*H" {
            return Some(simple("M*A*S*H | series finale".into(), "Result", "February 28, 1983 (\"Goodbye, Farewell and Amen\", CBS)".into(), vec![pod("Note", "Note", text(sh.note))]));
        }
        return None;
    }
    // "how many The Twilight Zone episodes are there"
    if let Some(m) = re!(r"^(?:how many )?(?:episodes of )?(.+?)(?: episodes)? (?:are there|were there|were made|were produced)$|^(?:number of )?episodes (?:of|in) (.+)$").captures(l) {
        let sh = find_show(m.get(1).or(m.get(2))?.as_str())?;
        return Some(simple(format!("{} | number of episodes", sh.title), "Result", format!("{} episodes in {} seasons ({}, {})", sh.episodes, sh.seasons, sh.network, sh.years), vec![pod("Note", "Note", text(sh.note))]));
    }
    // "Law and Order tv franchise"
    if re!(r"^law (?:and|&) order(?: tv)? (?:franchise|series|shows)$").is_match(l) {
        let rows: Vec<Vec<Content>> = [("Law & Order", "1990–2010; 2022–"), ("Law & Order: Special Victims Unit", "1999–"), ("Law & Order: Criminal Intent", "2001–2011"), ("Law & Order: Trial by Jury", "2005–2006"), ("Law & Order: LA", "2010–2011"), ("Law & Order True Crime", "2017"), ("Law & Order: Organized Crime", "2021–2025")].iter().map(|(a, b)| vec![text(*a), text(*b)]).collect();
        return Some(simple("Law & Order (TV franchise)".into(), "Result", "an NBC crime-drama franchise created by Dick Wolf, begun in 1990".into(), vec![pod("Table", "Series", table(Some(vec!["series", "aired"]), rows))]));
    }
    // "Spock TV shows"
    if re!(r"^(?:mr\.? )?spock (?:tv shows|television shows|tv series|shows)$|^(?:tv shows|shows) (?:with|featuring) (?:mr\.? )?spock$").is_match(l) {
        let rows: Vec<Vec<Content>> = [("Star Trek", "1966–1969", "Leonard Nimoy"), ("Star Trek: The Animated Series", "1973–1974", "Leonard Nimoy (voice)"), ("Star Trek: The Next Generation (\"Unification\", 2 episodes)", "1991", "Leonard Nimoy"), ("Star Trek: Discovery (season 2)", "2019", "Ethan Peck"), ("Star Trek: Strange New Worlds", "2022–", "Ethan Peck")].iter().map(|(a, b, c)| vec![text(*a), text(*b), text(*c)]).collect();
        return Some(simple("Spock | television series".into(), "Result", "Star Trek | Star Trek: The Animated Series | Star Trek: The Next Generation | Star Trek: Discovery | Star Trek: Strange New Worlds".into(), vec![pod("Table", "Appearances", table(Some(vec!["series", "years", "played by"]), rows))]));
    }
    // "who has played Dumbledore", "who has played James Bond"
    if let Some(m) = re!(r"^who (?:has|have) played (.+)$|^actors? who (?:have )?played (.+)$").captures(l) {
        let who = m.get(1).or(m.get(2))?.as_str().trim_start_matches("albus ");
        let rows: Vec<(&str, &str)> = match who {
            "dumbledore" => vec![("Richard Harris", "Harry Potter and the Philosopher's Stone (2001), Chamber of Secrets (2002)"), ("Michael Gambon", "Prisoner of Azkaban (2004) through Deathly Hallows – Part 2 (2011)"), ("Jude Law", "Fantastic Beasts: The Crimes of Grindelwald (2018), The Secrets of Dumbledore (2022), as the younger Dumbledore"), ("Toby Regbo", "Deathly Hallows – Part 1 (2010), as the teenage Dumbledore")],
            "james bond" | "bond" | "007" => vec![("Sean Connery", "1962–1967, 1971"), ("George Lazenby", "1969"), ("Roger Moore", "1973–1985"), ("Timothy Dalton", "1987–1989"), ("Pierce Brosnan", "1995–2002"), ("Daniel Craig", "2006–2021")],
            _ => return None,
        };
        return Some(simple(format!("actors who played {}", cap_words(who)), "Actors", rows.iter().map(|r| r.0).collect::<Vec<_>>().join(" | "), vec![pod("Table", "Roles", table(Some(vec!["actor", "films"]), rows.iter().map(|(a, b)| vec![text(*a), text(*b)]).collect()))]));
    }
    // "video games developed by Infinity Ward", "PS3 video games by Naughty Dog"
    if let Some(m) = re!(r"^(?:(ps3|ps4|ps2|ps1|playstation 3|playstation 4) )?(?:video )?games (?:developed |made )?by (.+)$").captures(l) {
        let (_, list) = DEVELOPED.iter().find(|(n, _)| *n == m[2].trim())?;
        let plat = m.get(1).map(|x| x.as_str().replace("playstation ", "ps"));
        let v: Vec<&(&str, i32, &str)> = list.iter().filter(|g| plat.as_ref().is_none_or(|p| g.2.to_lowercase().split(", ").any(|x| x.starts_with(p.as_str())))).collect();
        if v.is_empty() {
            return None;
        }
        return Some(simple(format!("{}video games developed by {}", plat.map(|p| format!("{} ", p.to_uppercase())).unwrap_or_default(), cap_words(m[2].trim())), "Games", v.iter().map(|g| format!("{} ({})", g.0, g.1)).collect::<Vec<_>>().join(" | "), vec![pod("Table", "Games", table(Some(vec!["title", "year", "platforms"]), v.iter().map(|g| vec![text(g.0), text(g.1.to_string()), text(g.2)]).collect()))]));
    }
    let find_game = |s: &str| GAMES.iter().find(|g| g.title.to_lowercase() == s || g.aliases.contains(&s));
    // "Who published Zelda 2?", "platforms for NBA Jam game", "release date of Call of Duty 4 for the PS3"
    if let Some(m) = re!(r"^who (published|developed|made) (.+)$|^(platforms?|publisher|developer|release date) (?:for|of) (.+?)(?: game)?(?: for the (?:ps3|playstation 3|xbox 360))?$|^what'?s? (?:is )?the (release date) of (.+?)(?: for the (?:ps3|playstation 3|xbox 360))?$").captures(l) {
        let (p, who) = if let Some(p) = m.get(1) { (p.as_str(), &m[2]) } else if let Some(p) = m.get(3) { (p.as_str(), &m[4]) } else { (&m[5], &m[6]) };
        let g = find_game(who.trim())?;
        let (label, v) = match p {
            "published" | "publisher" => ("Publisher", g.publisher),
            "developed" | "made" | "developer" => ("Developer", g.developer),
            "release date" => ("Release date", g.released),
            _ => ("Platforms", g.platforms),
        };
        return Some(simple(format!("{} | {}", g.title, label.to_lowercase()), label, v.into(), vec![game_pod(g)]));
    }
    let g = find_game(l)?;
    Some(simple(format!("{} (video game)", g.title), "Basic information", format!("developed by {}, published by {}; released {}", g.developer, g.publisher, g.released), vec![game_pod(g)]))
}

fn game_pod(g: &Game) -> Pod {
    let mut r = vec![vec![text("developer"), text(g.developer)], vec![text("publisher"), text(g.publisher)], vec![text("release date"), text(g.released)], vec![text("platforms"), text(g.platforms)]];
    if !g.note.is_empty() {
        r.push(vec![text("note"), text(g.note)]);
    }
    pod("Table", "Details", table(None, r))
}

/// "total hours of programming for the Simpsons": episodes × typical running time.
fn show_hours(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the )?total (?:hours|runtime|running time|length|viewing time|time)(?: of programming| of episodes| of all episodes)? (?:for|of|in) (?:all )?(?:the )?(.+?)(?: episodes)?$").captures(l)?;
    let name = m[1].trim();
    let s = SHOWS.iter().find(|x| x.title.to_lowercase() == name || x.title.to_lowercase() == format!("the {name}") || x.aliases.contains(&name))?;
    let minutes: f64 = match s.title {
        "The Simpsons" => 22.0,
        "The Flintstones" => 25.0,
        "M*A*S*H" => 25.0,
        "The Walking Dead" => 44.0,
        "Game of Thrones" => 57.0,
        _ => return None,
    };
    let total = s.episodes as f64 * minutes;
    Some(simple(format!("{} | total running time", s.title), "Total running time", format!("about {} hours ({} days)", (total / 60.0).round(), (total / 1440.0 * 10.0).round() / 10.0), vec![pod("Inputs", "Estimate", table(None, vec![
        vec![text("episodes"), text(s.episodes.to_string())],
        vec![text("typical episode length"), text(format!("about {minutes} minutes without commercials"))],
    ]))]))
}

/// "Flintstones vs the Simpsons"
fn shows(l: &str) -> Option<Answer> {
    let parts: Vec<&str> = re!(r"\s*(?:,|\bvs\.?\s|\bversus\b)\s*").split(l).filter(|x| !x.trim().is_empty()).collect();
    if parts.is_empty() {
        return None;
    }
    let v: Option<Vec<&Show>> = parts.iter().map(|p| SHOWS.iter().find(|x| x.title.to_lowercase() == p.trim() || x.aliases.contains(&p.trim()))).collect();
    let v = v?;
    if v.len() == 1 && !matches!(l, "the twilight zone" | "the flintstones" | "the simpsons" | "game of thrones" | "the walking dead" | "m*a*s*h") {
        return None;
    }
    let rows = v.iter().map(|s| vec![text(s.title), text(s.network), text(s.years), text(s.seasons.to_string()), text(s.episodes.to_string())]).collect();
    Some(simple(v.iter().map(|s| s.title).collect::<Vec<_>>().join(" vs "), "Television series", v.iter().map(|s| format!("{} ({}, {})", s.title, s.network, s.years)).collect::<Vec<_>>().join(" | "), vec![pod("Table", "Comparison", table(Some(vec!["series", "network", "aired", "seasons", "episodes"]), rows))]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim().replace('’', "'");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let l = l.as_str();
    if l == "mario kart platform" || l == "mario kart platforms" {
        return Some(simple("Mario Kart (video game series) | platform".into(), "Platforms", "Super Mario Kart (1992) on the Super Nintendo Entertainment System; later games on the Nintendo 64, Game Boy Advance, GameCube, Nintendo DS, Wii, Nintendo 3DS, Wii U, Nintendo Switch and Switch 2, plus arcade and mobile versions".into(), vec![]));
    }
    if re!(r"^what was the age of leonardo(?: da vinci)? when (?:the )?mona lisa was (?:painted|begun|started)$|^how old was leonardo(?: da vinci)? when he (?:painted|began) (?:the )?mona lisa$").is_match(l) {
        return Some(simple("Leonardo da Vinci | age when Mona Lisa was begun".into(), "Result", "about 51 years (Leonardo, born April 15, 1452, began the Mona Lisa in about 1503 and kept working on it until about 1517)".into(), vec![]));
    }
    if let Some(a) = tv_and_games(l).or_else(|| show_hours(l)).or_else(|| shows(l)).or_else(|| franchise(l)).or_else(|| people(l)).or_else(|| box_office(l)).or_else(|| film_props(l)) {
        return Some(a);
    }
    music(l)
}
